use std::sync::Arc;

use lazy_static::lazy_static;
use serde_json::Value;

use crate::socket::session::Session;
use crate::socket::types::SocketResponse;

use super::CommandProcessor;

/// 这是一个Echo command处理器的Demo
pub struct EchoCommandProcessor {}

lazy_static! {
    static ref INSTANCE: EchoCommandProcessor = EchoCommandProcessor::new();
}

impl EchoCommandProcessor {
    fn new() -> Self {
        EchoCommandProcessor {}
    }

    pub fn static_new() -> &'static Self {
        return &INSTANCE;
    }
}

impl CommandProcessor for EchoCommandProcessor {
    fn process(&self, session: Arc<Session>, command: &crate::command::Command) {
        let mut resoponse = SocketResponse::<Value>::from(command);
        if let Some(args) = &command.args {
            resoponse = resoponse.set_data(args.clone())
        }
        _ = session.send_response(resoponse);
    }

    fn matches(&'static self, command: &crate::command::Command) -> bool {
        return command.op.as_str() == "echo";
    }
}
