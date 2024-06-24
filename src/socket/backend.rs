use std::borrow::Cow;
use std::sync::Arc;

use axum::extract::ws::{CloseFrame, Message};
use dashmap::DashMap;
use lib_utils::times::current_timestamp;

use crate::{command::CommandDispatcher, socket::session::Session};

pub struct WebSocketBackend {
    session_map: DashMap<String, Arc<Session>>,
    command_dispatcher: CommandDispatcher,
}

impl WebSocketBackend {
    pub fn new() -> &'static Self {
        let backend = WebSocketBackend {
            session_map: DashMap::new(),
            command_dispatcher: CommandDispatcher::new(),
        };
        return Box::leak(Box::new(backend));
    }

    pub fn add_session(&self, session: Arc<Session>) {
        let session_id = session.get_session_id();
        self.session_map.insert(session_id, session);
    }

    pub fn remove_session<T: AsRef<str>>(&self, session_id: T) {
        let id = session_id.as_ref();
        self.session_map.remove(id);
    }

    pub fn check_session_status(&'static self) {
        tokio::spawn(async move {
            loop {
                let timestamp = current_timestamp();
                for session in self.session_map.iter() {
                    if session.is_expired(timestamp) {
                        let _ = session.send_message(Message::Close(Some(CloseFrame {
                            code: axum::extract::ws::close_code::NORMAL,
                            reason: Cow::from("expired"),
                        })));
                    }
                }
            }
        });
    }

    pub fn get_command_dispatcher(&self) -> &CommandDispatcher {
        return &self.command_dispatcher;
    }
}
