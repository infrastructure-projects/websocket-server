use std::sync::Arc;

use echo::EchoCommandProcessor;

use crate::socket::session::Session;

use super::Command;

mod echo;

/// CommandProcessor trait 用于处理和匹配命令.self被声明为静态声明周期
trait CommandProcessor: Send + Sync {
    /// 处理命令
    fn process(&'static self, session: Arc<Session>, command: &Command);
    /// 检查命令能否被此处理器处理
    fn matches(&'static self, command: &Command) -> bool;
}

pub struct CommandDispatcher {
    processors: Vec<&'static dyn CommandProcessor>,
}

impl CommandDispatcher {
    pub fn new() -> Self {
        let mut processors: Vec<&'static dyn CommandProcessor> = Vec::new();
        processors.push(EchoCommandProcessor::static_new());
        CommandDispatcher { processors }
    }
}

impl CommandDispatcher {
    pub fn dispatch(&self, session: Arc<Session>, command: &Command) {
        for processor in self.processors.iter() {
            if processor.matches(command) {
                processor.process(session.clone(), command);
                break;
            }
        }
    }
}
