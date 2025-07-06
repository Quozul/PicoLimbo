#[derive(Debug, thiserror::Error)]
#[error("error decoding packet")]
pub struct DecodePacketError;
