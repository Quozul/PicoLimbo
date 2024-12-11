use crate::prelude::SerializePacketData;
use nbt::prelude::Nbt;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum NbtEncodeError {
    #[error("failed to encode nbt; error={0}")]
    Io(std::io::Error),
    #[error("failed to encode nbt")]
    Infallible,
}

impl From<std::convert::Infallible> for NbtEncodeError {
    fn from(_: std::convert::Infallible) -> Self {
        NbtEncodeError::Infallible
    }
}

impl From<std::io::Error> for NbtEncodeError {
    fn from(error: std::io::Error) -> Self {
        NbtEncodeError::Io(error)
    }
}

impl SerializePacketData for Nbt {
    type Error = NbtEncodeError;

    fn encode(&self, bytes: &mut Vec<u8>) -> Result<(), Self::Error> {
        let nbt_bytes = self.to_bytes();
        // FIXME: Should send length?
        // VarInt::new(nbt_bytes.len() as i32).encode(bytes)?;
        bytes.extend_from_slice(&nbt_bytes);
        Ok(())
    }
}
