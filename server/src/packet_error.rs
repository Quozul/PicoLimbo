use protocol::prelude::handshaking::data::state::State;
use protocol::prelude::DecodePacketError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PacketError {
    #[error("unknown packet received; state={state:?}, packet_id=0x{packet_id:02x}")]
    Unknown { state: State, packet_id: u8 },
    #[error("error decoding packet {0}")]
    Decode(#[from] DecodePacketError),
}

impl PacketError {
    pub fn new(state: State, packet_id: u8) -> PacketError {
        PacketError::Unknown { state, packet_id }
    }
}
