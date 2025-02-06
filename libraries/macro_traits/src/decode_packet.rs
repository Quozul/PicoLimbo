use thiserror::Error;

#[derive(Debug, Error)]
#[error("error decoding packet")]
pub struct DecodePacketError;

pub trait DecodePacket: Sized {
    fn decode(
        bytes: &[u8],
        protocol_version: &protocol_version::ProtocolVersion,
    ) -> Result<Self, DecodePacketError>;
}
