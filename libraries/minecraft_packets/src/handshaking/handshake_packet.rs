use crate::handshaking::data::state::{State, UnknownStateError};
use data_types::prelude::{DecodePacketField, VarInt};
use macros::{PacketIn, packet_id};
use protocol_version::ProtocolVersion;

#[derive(Debug, PacketIn)]
#[packet_id(0x00)]
pub struct HandshakePacket {
    protocol: VarInt,
    hostname: String,
    port: u16,
    next_state: VarInt,
}

impl HandshakePacket {
    pub fn get_next_state(&self) -> Result<State, UnknownStateError> {
        match self.next_state.value() {
            1 => Ok(State::Status),
            2 => Ok(State::Login),
            3 => Ok(State::Transfer), // Added as of v1_20_X // FIXME: Check if this is correct
            _ => Err(UnknownStateError(self.next_state.value())),
        }
    }

    pub fn get_protocol_version(&self) -> ProtocolVersion {
        self.protocol.clone().into()
    }
}
