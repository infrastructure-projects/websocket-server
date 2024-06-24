use serde::Deserialize;
use serde_json::Value;

pub use processor::CommandDispatcher;

mod processor;

#[derive(Debug, Deserialize)]
pub struct Command {
    pub op: String,
    pub channel: Option<String>,
    pub args: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "requestId")]
    pub request_id: Option<String>,
}
