use crate::RegistryManager;
use crate::registry_provider::shared::get_registry_keys;
use pico_identifier::Identifier;
use pico_nbt::{CompressionType, NbtOptions};
use protocol_version::protocol_version::ProtocolVersion;
use std::borrow::Cow;

#[derive(Clone)]
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
    let registries = get_registry_keys(protocol_version)?;

    Ok(registries
        .iter()
        .filter_map(|registry_keys| registry_manager.try_get(registry_keys))
        // Registries that only exist to carry tags have no entry directory and load
        // empty. Sending an empty Registry Data packet for them carries no
        // information, and clients that do not expect the registry to be synced at
        // all can fail hard on it (Geyser throws on `minecraft:block`).
        .filter(|registry| registry.has_entries())
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
