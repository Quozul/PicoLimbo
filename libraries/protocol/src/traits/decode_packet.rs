use crate::prelude::DecodePacketError;

pub trait DecodePacket: Sized {
    fn decode(bytes: &[u8]) -> Result<Self, DecodePacketError>;
}
