use data_types::prelude::EncodePacketField;
use macros::{PacketOut, packet_id};
use crate::status::data::status_response::StatusResponse;

#[derive(Debug, PacketOut)]
#[packet_id(0x00)]
pub struct StatusResponsePacket {
    json_response: String,
}

impl StatusResponsePacket {
    pub fn from_status_response(status_response: &StatusResponse) -> Self {
        let json_response = serde_json::to_string(status_response).unwrap();
        StatusResponsePacket { json_response }
    }
}
