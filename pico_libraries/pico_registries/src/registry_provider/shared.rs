use crate::data::registry_entry::RegistryEntry;
use crate::registry_provider::Dimension;
use crate::{RegistryKeys, RegistryManager, RegistryManagerBuilder};
use pico_nbt2::NbtOptions;
use protocol_version::protocol_version::ProtocolVersion;
use serde::Serialize;
use std::borrow::Cow;
use std::path::Path;

pub fn load_registry_manager(
    base_path: &Path,
    protocol_version: ProtocolVersion,
    registries: &[RegistryKeys],
) -> crate::Result<RegistryManager> {
    crate::Error::incompatible_version(
        protocol_version,
        ProtocolVersion::V1_16,
        ProtocolVersion::latest(),
    )?;

    let resource_root = base_path.join(protocol_version.data().to_string());

    Ok(RegistryManager::builder()
        .register_all(registries)
        .load_from_resource_path(&resource_root))
}

pub fn get_registry_keys(protocol_version: ProtocolVersion) -> crate::Result<Vec<RegistryKeys>> {
    crate::Error::incompatible_version(
        protocol_version,
        ProtocolVersion::V1_16,
        ProtocolVersion::latest(),
    )?;
    Ok(RegistryManagerBuilder::DEFAULT_REGISTRIES
        .iter()
        .filter(|key| {
            key.is_mandatory()
                && key.get_minimum_version().is_some_and(|minimum_version| {
                    protocol_version.is_after_inclusive(minimum_version)
                })
        })
        .cloned()
        .collect())
}

pub fn get_dimension(
    registry_manager: &RegistryManager,
    dimension_identifier: Dimension,
) -> crate::Result<&RegistryEntry> {
    registry_manager
        .get(&RegistryKeys::DimensionType)?
        .try_get(&dimension_identifier.identifier())
        .ok_or(crate::Error::UnknownRegistryEntry)
}

pub fn encode_nameless_compound_to_bytes<T: Serialize>(
    protocol_version: ProtocolVersion,
    value: &T,
) -> pico_nbt2::Result<Cow<'static, [u8]>> {
    let is_nameless = protocol_version.is_after_inclusive(ProtocolVersion::V1_20_2);
    let options = NbtOptions::new().nameless_root(is_nameless);
    let name = if is_nameless { None } else { Some("") };
    let mut bytes = Vec::new();
    pico_nbt2::to_writer_with_options(&mut bytes, &value, name, options)?;
    Ok(Cow::Owned(bytes))
}
