use crate::prelude::*;
use nbt::prelude::Nbt;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum NbtEncodeError {
    #[error("failed to encode nbt; error={0}")]
    Io(#[from] std::io::Error),
    #[error("failed to encode nbt")]
    Infallible(#[from] std::convert::Infallible),
}

impl EncodePacketField for Nbt {
    type Error = NbtEncodeError;

    fn encode(&self, bytes: &mut Vec<u8>) -> Result<(), Self::Error> {
        let nbt_bytes = self.to_bytes();
        bytes.extend_from_slice(&nbt_bytes);
        Ok(())
    }
}
