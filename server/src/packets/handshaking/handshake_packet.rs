use crate::server::protocol_version::ProtocolVersion;
use crate::state::State;
use protocol::prelude::*;
use thiserror::Error;

#[derive(Debug, PacketIn)]
#[packet_id("handshake/serverbound/minecraft:intention")]
pub struct HandshakePacket {
    protocol: VarInt,
    hostname: String,
    port: u16,
    next_state: VarInt,
}

#[derive(Error, Debug)]
#[error("unknown state {0}")]
pub struct UnknownStateError(i32);

impl HandshakePacket {
    pub fn get_next_state(&self) -> Result<State, UnknownStateError> {
        let state = self.next_state.value();
        match state {
            1 => Ok(State::Status),
            2 => Ok(State::Login),
            3 => Ok(State::Transfer),
            _ => Err(UnknownStateError(state)),
        }
    }

    pub fn get_protocol(&self) -> ProtocolVersion {
        ProtocolVersion::from(self.protocol.value())
    }
}
