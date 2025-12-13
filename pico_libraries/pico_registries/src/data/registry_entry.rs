use crate::data::registry_entry_value::{DimensionType, RegistryEntryValue};
use crate::data::registry_key::RegistryKey;
use pico_nbt2::Value;
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Serialize)]
pub struct RegistryEntry {
    value: RegistryEntryValue,
    #[serde(skip_serializing)]
    raw_value: Value,
    registry_key: RegistryKey,
    protocol_id: u32,
}

#[derive(Debug, Error)]
pub enum RegistryEntryError {
    #[error("this registry entry is not of the expected type")]
    NotOfType,
}

impl RegistryEntry {
    pub const fn new(
        value: RegistryEntryValue,
        raw_value: Value,
        registry_key: RegistryKey,
        protocol_id: u32,
    ) -> Self {
        Self {
            value,
            raw_value,
            registry_key,
            protocol_id,
        }
    }

    pub const fn get_dimension(&self) -> Result<&DimensionType, RegistryEntryError> {
        match self.value {
            RegistryEntryValue::DimensionType(ref dimension) => Ok(dimension),
            _ => Err(RegistryEntryError::NotOfType),
        }
    }

    pub const fn get_protocol_id(&self) -> u32 {
        self.protocol_id
    }

    pub const fn get_registry_key(&self) -> &RegistryKey {
        &self.registry_key
    }
}
