use crate::prelude::EncodePacketField;
use crate::protocol_version::ProtocolVersion;
use nbt::prelude::{Nbt, NbtFeatures};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum NbtEncodeError {
    #[error("failed to encode pico_nbt; error={0}")]
    Io(std::io::Error),
    #[error("failed to encode pico_nbt")]
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

impl EncodePacketField for Nbt {
    type Error = NbtEncodeError;

    fn encode(&self, bytes: &mut Vec<u8>, protocol_version: u32) -> Result<(), Self::Error> {
        let nbt_features = nbt_features_from_protocol_version(protocol_version);
        let nbt_bytes = self.to_bytes(nbt_features);
        bytes.extend_from_slice(&nbt_bytes);
        Ok(())
    }
}

fn nbt_features_from_protocol_version(protocol_version: u32) -> NbtFeatures {
    let mut builder = NbtFeatures::builder();
    if protocol_version >= ProtocolVersion::V1_20_2.version_number() {
        builder.nameless();
    };
    if protocol_version >= ProtocolVersion::V1_21_5.version_number() {
        builder.dynamic_lists();
    };
    builder.build()
}
