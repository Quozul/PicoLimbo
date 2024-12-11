use thiserror::Error;

#[derive(Debug, Error)]
#[error("error decoding packet")]
pub struct DecodePacketError;
