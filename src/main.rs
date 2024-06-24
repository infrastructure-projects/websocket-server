use socket::WebSocketServer;

pub mod command;
pub mod socket;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let server = WebSocketServer::new();
    server.run().await;
}
