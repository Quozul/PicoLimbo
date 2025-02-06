use crate::packet_error::PacketError;
use protocol::prelude::handshaking::data::state::State;
use protocol::prelude::login::login_acknowledged_packet::LoginAcknowledgedPacket;
use protocol::prelude::login::login_start_packet::LoginStartPacket;
use protocol::prelude::{DecodePacket, PacketId, ProtocolVersion, Uuid};

pub enum LoginResult {
    Login(Uuid, String),
    LoginAcknowledged,
}

pub fn handle_login_state(
    packet_id: u8,
    payload: &[u8],
    protocol_version: &ProtocolVersion,
) -> Result<LoginResult, PacketError> {
    if LoginStartPacket::is_packet(packet_id, protocol_version) {
        let packet = LoginStartPacket::decode(payload, protocol_version)?;
        Ok(LoginResult::Login(packet.player_uuid, packet.name))
    } else if LoginAcknowledgedPacket::is_packet(packet_id, protocol_version) {
        LoginAcknowledgedPacket::decode(payload, protocol_version)?;
        Ok(LoginResult::LoginAcknowledged)
    } else {
        Err(PacketError::new(State::Login, packet_id))
    }
}
