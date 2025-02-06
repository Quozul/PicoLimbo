use crate::packet_error::PacketError;
use protocol::prelude::handshaking::data::state::State;
use protocol::prelude::handshaking::handshake_packet::HandshakePacket;
use protocol::prelude::{DecodePacket, PacketId, ProtocolVersion};

/// Returns the next state
pub fn handle_handshake_state(
    packet_id: u8,
    payload: &[u8],
    protocol_version: &ProtocolVersion,
) -> Result<HandshakePacket, PacketError> {
    if HandshakePacket::is_packet(packet_id, protocol_version) {
        Ok(HandshakePacket::decode(payload, protocol_version)?)
    } else {
        Err(PacketError::new(State::Handshake, packet_id))
    }
}
