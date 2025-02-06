use crate::packet_error::PacketError;
use protocol::prelude::handshaking::data::state::State;
use protocol::prelude::status::ping_request_packet::PingRequestPacket;
use protocol::prelude::status::status_request_packet::StatusRequestPacket;
use protocol::prelude::{DecodePacket, PacketId, ProtocolVersion};

pub enum StatusResult {
    Status,
    Ping(i64),
}

pub fn handle_status_state(
    packet_id: u8,
    payload: &[u8],
    protocol_version: &ProtocolVersion,
) -> Result<StatusResult, PacketError> {
    if StatusRequestPacket::is_packet(packet_id, protocol_version) {
        StatusRequestPacket::decode(payload, protocol_version)?;
        Ok(StatusResult::Status)
    } else if PingRequestPacket::is_packet(packet_id, protocol_version) {
        let packet = PingRequestPacket::decode(payload, protocol_version)?;
        Ok(StatusResult::Ping(packet.timestamp))
    } else {
        Err(PacketError::new(State::Status, packet_id))
    }
}
