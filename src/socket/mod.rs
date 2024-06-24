use std::sync::Arc;

use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    response::Response,
    routing::get,
};
use flume::Receiver;
use futures::stream::SplitSink;
use futures::{stream::SplitStream, SinkExt, StreamExt};
use serde_json::from_str;
use tracing::log;
use uuid::Uuid;

use session::Session;
use types::SocketResponse;

use crate::{command::Command, socket::backend::WebSocketBackend};

mod backend;
pub mod session;
pub mod types;

pub struct WebSocketServer {
    backend: &'static WebSocketBackend,
}

impl WebSocketServer {
    pub fn new() -> &'static Self {
        let backend = WebSocketBackend::new();
        backend.check_session_status();
        let server = WebSocketServer { backend };
        Box::leak(Box::new(server))
    }

    pub async fn run(&'static self) {
        log::info!("[Server]正在启动WebSocket服务...");
        let app = axum::Router::new()
            .route("/connect", get(WebSocketServer::handler))
            .with_state(self.backend);

        let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
        log::info!("[Server]WebSocket服务启动完成.访问地址: ws://127.0.0.1:8000/connect");
        axum::serve(listener, app).await.unwrap();
    }

    async fn handler(
        ws: WebSocketUpgrade,
        State(state): State<&'static WebSocketBackend>,
    ) -> Response {
        ws.on_upgrade(|socket| Self::handle_socket(socket, state))
    }

    async fn handle_socket(socket: WebSocket, backend: &'static WebSocketBackend) {
        let (sink, receiver) = socket.split();
        let (tx, rx) = flume::bounded::<Message>(128);
        let session = Arc::new(Session::new(tx.clone()));
        backend.add_session(session.clone());

        let mut send_task = tokio::spawn(async {
            Self::sink_event_loop(rx, sink).await;
        });

        let ss = session.clone();
        let mut recv_task = tokio::spawn(async {
            Self::receive_event_loop(receiver, ss, backend).await;
        });
        tokio::select! {
            _ = (&mut send_task) => {
                recv_task.abort();
            },
            _ = (&mut recv_task) => {
                send_task.abort();
            }
        }
        drop(tx);
        backend.remove_session(session.get_session_id());
    }

    async fn sink_event_loop(rx: Receiver<Message>, mut sink: SplitSink<WebSocket, Message>) {
        while let Ok(msg) = rx.recv_async().await {
            if sink.send(msg).await.is_err() {
                break;
            }
        }
    }

    async fn receive_event_loop(
        mut receiver: SplitStream<WebSocket>,
        session: Arc<Session>,
        backend: &WebSocketBackend,
    ) {
        while let Some(msg) = receiver.next().await {
            if let Ok(msg) = msg {
                match msg {
                    Message::Text(text) => {
                        session.refresh_active_status();
                        let msg = text.trim();
                        // 是否是ping消息
                        if msg.to_lowercase().as_str() == "ping" {
                            let _ = session.send_message("pong");
                            continue;
                        }
                        if let Ok(mut command) = from_str::<Command>(msg) {
                            if command.request_id.is_none() {
                                command.request_id = Some(Uuid::new_v4().to_string())
                            }
                            backend
                                .get_command_dispatcher()
                                .dispatch(session.clone(), &command);
                        } else {
                            let response = SocketResponse::<String>::default()
                                .with_bad_command();
                            let _ = session.send_response(response);
                        }
                    }
                    Message::Ping(msg) => {
                        session.refresh_active_status();
                        let _ = session.send_message(Message::Pong(msg));
                    }
                    Message::Pong(msg) => {
                        session.refresh_active_status();
                        let _ = session.send_message(Message::Ping(msg));
                    }
                    Message::Close(status) => {
                        let _ = session.send_message(Message::Close(status));
                        backend.remove_session(session.get_session_id());
                        break;
                    }
                    Message::Binary(_) => {
                        let _ = session
                            .send_message(Message::from("Binary data type is not supported"));
                    }
                }
            }
        }
    }
}
