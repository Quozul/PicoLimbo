use crate::status::data::status_response::StatusResponse;
use minecraft_protocol::prelude::*;

#[derive(Debug, PacketOut)]
pub struct StatusResponsePacket {
    json_response: String,
}

impl StatusResponsePacket {
    pub fn from_status_response(status_response: &StatusResponse) -> Self {
        let json_response = serde_json::to_string(status_response).unwrap();
        StatusResponsePacket { json_response }
    }

    pub fn status_response(&self) -> serde_json::Result<StatusResponse> {
        serde_json::from_str(self.json_response.as_str())
    }
}
