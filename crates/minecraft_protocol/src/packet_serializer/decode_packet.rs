use crate::prelude::DecodePacketError;

pub trait DecodePacket: Sized {
    fn decode(bytes: &[u8], protocol_version: i32) -> Result<Self, DecodePacketError>;
}
