//------------------------------------
//        WebSocket Response
//------------------------------------

use serde::Serialize;

use crate::command::Command;

#[derive(Debug, Serialize)]
pub struct SocketResponse<T: Serialize> {
    code: String,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    op: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    channel: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "requestId")]
    pub request_id: Option<String>,
}

impl<T: Serialize> Default for SocketResponse<T> {
    fn default() -> SocketResponse<T> {
        SocketResponse {
            op: None,
            data: None,
            channel: None,
            code: "200".to_string(),
            message: "success".to_string(),
            request_id: None,
        }
    }
}

impl<T: Serialize> From<&Command> for SocketResponse<T> {
    fn from(command: &Command) -> Self {
        let mut resp = SocketResponse::default();
        resp.op = Some(command.op.clone());
        resp.channel = command.channel.clone();
        resp.request_id = command.request_id.clone();
        return resp;
    }
}

impl<T: Serialize> SocketResponse<T> {
    pub fn set_data(mut self, data: T) -> Self {
        self.data = Some(data);
        self
    }

    pub fn with_server_error(mut self) -> Self {
        self.code = "500".to_string();
        self.message = "server error".to_string();
        self
    }

    pub fn with_success(mut self) -> Self {
        self.code = "200".to_string();
        self.message = "success".to_string();
        self
    }

    pub fn with_bad_command(mut self) -> Self {
        self.code = "4001".to_string();
        self.message = "bad command".to_string();
        self
    }

    pub fn with_invalid_params(mut self) -> Self {
        self.code = "4006".to_string();
        self.message = "invalid params".to_string();
        self
    }

    pub fn as_json(&self) -> anyhow::Result<String> {
        let data = serde_json::to_string(self)?;
        Ok(data)
    }
}
