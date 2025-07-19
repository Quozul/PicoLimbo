use crate::prelude::*;
use thiserror::Error;

#[derive(Debug)]
pub struct RegistryEntry {
    pub entry_id: Identifier,
    /// Whether the entry has any data following.
    pub has_data: bool,
    /// Entry data. Only present if Has Data is true.
    pub nbt: Option<Nbt>,
}

#[derive(Debug, Error)]
pub enum RegistryEntryEncodeError {
    #[error("failed to encode identifier")]
    Identifier,
    #[error("failed to encode pico_nbt")]
    Infallible,
}

impl EncodePacketField for RegistryEntry {
    type Error = RegistryEntryEncodeError;

    fn encode(&self, bytes: &mut Vec<u8>, protocol_version: i32) -> Result<(), Self::Error> {
        self.entry_id
            .encode(bytes, protocol_version)
            .map_err(|_| RegistryEntryEncodeError::Identifier)?;
        self.has_data
            .encode(bytes, protocol_version)
            .map_err(|_| RegistryEntryEncodeError::Infallible)?;
        if let Some(nbt) = &self.nbt {
            nbt.encode(bytes, protocol_version)
                .map_err(|_| RegistryEntryEncodeError::Infallible)?;
        }
        Ok(())
    }
}
