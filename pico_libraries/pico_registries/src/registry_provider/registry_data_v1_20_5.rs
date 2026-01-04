use crate::RegistryManager;
use crate::registry_provider::shared::get_registry_keys;
use pico_identifier::Identifier;
use pico_nbt2::{CompressionType, NbtOptions};
use protocol_version::protocol_version::ProtocolVersion;
use std::borrow::Cow;

pub struct RegistryDataEntry {
    pub entry_id: Identifier,
    pub nbt_bytes: Cow<'static, [u8]>,
}

impl RegistryDataEntry {
    #[must_use]
    pub const fn new(entry_id: Identifier, nbt_bytes: Cow<'static, [u8]>) -> Self {
        Self {
            entry_id,
            nbt_bytes,
        }
    }
}

pub fn get_registry_data_v1_20_5(
    registry_manager: &RegistryManager,
    protocol_version: ProtocolVersion,
) -> crate::Result<Vec<(Identifier, Vec<RegistryDataEntry>)>> {
    crate::Error::incompatible_version(
        protocol_version,
        ProtocolVersion::V1_20_5,
        ProtocolVersion::latest(),
    )?;
    let registries = get_registry_keys(protocol_version)?;

    Ok(registries
        .iter()
        .filter_map(|registry_keys| registry_manager.try_get(registry_keys))
        .map(|registry| {
            let registry_entries = registry
                .get_entries()
                .iter()
                .flat_map(|entry| -> crate::Result<RegistryDataEntry> {
                    let bytes = entry.get_raw_value().to_byte(
                        CompressionType::None,
                        NbtOptions::new().nameless_root(true).dynamic_lists(true),
                        None,
                    )?;
                    let entry_id = entry.get_registry_key().get_value().clone();
                    Ok(RegistryDataEntry::new(entry_id, Cow::Owned(bytes)))
                })
                .collect();
            let registry_id = registry.get_registry_key().get_value().clone();
            (registry_id, registry_entries)
        })
        .collect())
}
