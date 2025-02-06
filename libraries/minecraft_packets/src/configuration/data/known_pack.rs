use data_types::prelude::{DecodePacketField, EncodePacketField, StringDecodingError};
use thiserror::Error;

#[derive(Debug)]
pub struct KnownPack {
    pub namespace: String,
    pub id: String,
    pub version: String,
}

impl Default for KnownPack {
    fn default() -> Self {
        Self {
            namespace: "minecraft".to_string(),
            id: "core".to_string(),
            version: "1.21.4".to_string(),
        }
    }
}

#[derive(Error, Debug)]
#[error("error while decoding a packet; error={0}")]
pub enum DecodePacketError {
    #[error("error while decoding a string")]
    String(#[from] StringDecodingError),
}

impl DecodePacketField for KnownPack {
    type Error = DecodePacketError;

    fn decode(bytes: &[u8], index: &mut usize) -> Result<Self, Self::Error> {
        let namespace = String::decode(bytes, index).map_err(DecodePacketError::String)?;
        let id = String::decode(bytes, index).map_err(DecodePacketError::String)?;
        let version = String::decode(bytes, index).map_err(DecodePacketError::String)?;

        Ok(Self {
            namespace,
            id,
            version,
        })
    }
}

impl EncodePacketField for KnownPack {
    type Error = std::convert::Infallible;

    fn encode(&self, bytes: &mut Vec<u8>) -> Result<(), Self::Error> {
        self.namespace.encode(bytes)?;
        self.id.encode(bytes)?;
        self.version.encode(bytes)?;
        Ok(())
    }
}
