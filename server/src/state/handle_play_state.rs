use crate::packet_error::PacketError;
use protocol::prelude::handshaking::data::state::State;
use protocol::prelude::play::client_tick_end_packet::ClientTickEndPacket;
use protocol::prelude::play::server_bound_keep_alive_packet::ServerBoundKeepAlivePacket;
use protocol::prelude::play::set_player_position_packet::{
    SetPlayerPositionAndRotationPacket, SetPlayerPositionPacket, SetPlayerRotationPacket,
};
use protocol::prelude::{DecodePacket, PacketId, ProtocolVersion};

pub enum PlayResult {
    UpdatePositionAndRotation {
        x: Option<f64>,
        y: Option<f64>,
        z: Option<f64>,
        yaw: Option<f32>,
        pitch: Option<f32>,
    },
    Nothing,
}

impl From<SetPlayerPositionAndRotationPacket> for PlayResult {
    fn from(packet: SetPlayerPositionAndRotationPacket) -> Self {
        PlayResult::UpdatePositionAndRotation {
            x: Some(packet.x),
            y: Some(packet.y),
            z: Some(packet.z),
            yaw: Some(packet.yaw),
            pitch: Some(packet.pitch),
        }
    }
}

impl From<SetPlayerRotationPacket> for PlayResult {
    fn from(packet: SetPlayerRotationPacket) -> Self {
        PlayResult::UpdatePositionAndRotation {
            x: None,
            y: None,
            z: None,
            yaw: Some(packet.yaw),
            pitch: Some(packet.pitch),
        }
    }
}

impl From<SetPlayerPositionPacket> for PlayResult {
    fn from(packet: SetPlayerPositionPacket) -> Self {
        PlayResult::UpdatePositionAndRotation {
            x: Some(packet.x),
            y: Some(packet.y),
            z: Some(packet.z),
            yaw: None,
            pitch: None,
        }
    }
}

impl From<ServerBoundKeepAlivePacket> for PlayResult {
    fn from(_: ServerBoundKeepAlivePacket) -> Self {
        PlayResult::Nothing
    }
}

impl From<ClientTickEndPacket> for PlayResult {
    fn from(_: ClientTickEndPacket) -> Self {
        PlayResult::Nothing
    }
}

pub fn handle_play_state(
    packet_id: u8,
    payload: &[u8],
    protocol_version: &ProtocolVersion,
) -> Result<PlayResult, PacketError> {
    if ServerBoundKeepAlivePacket::is_packet(packet_id, protocol_version) {
        let packet = ServerBoundKeepAlivePacket::decode(payload, protocol_version)?;
        Ok(packet.into())
    } else if ClientTickEndPacket::is_packet(packet_id, protocol_version) {
        let packet = ClientTickEndPacket::decode(payload, protocol_version)?;
        Ok(packet.into())
    } else if SetPlayerPositionAndRotationPacket::is_packet(packet_id, protocol_version) {
        let packet = SetPlayerPositionAndRotationPacket::decode(payload, protocol_version)?;
        Ok(packet.into())
    } else if SetPlayerRotationPacket::is_packet(packet_id, protocol_version) {
        let packet = SetPlayerRotationPacket::decode(payload, protocol_version)?;
        Ok(packet.into())
    } else if SetPlayerPositionPacket::is_packet(packet_id, protocol_version) {
        let packet = SetPlayerPositionPacket::decode(payload, protocol_version)?;
        Ok(packet.into())
    } else {
        Err(PacketError::new(State::Play, packet_id))
    }
}
