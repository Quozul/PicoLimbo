use crate::data::registry_entry::RegistryEntry;
use crate::{RegistryKeys, RegistryManager, RegistryManagerBuilder};
use pico_identifier::Identifier;
use pico_nbt2::NbtOptions;
use protocol_version::protocol_version::ProtocolVersion;
use serde::Serialize;
use std::borrow::Cow;
use std::env;
use std::path::{Path, PathBuf};

fn find_data_path(sub_path: &str) -> Option<PathBuf> {
    let start_dir = env::var_os("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .or_else(|| env::current_dir().ok())?;

    let mut current = start_dir;

    loop {
        let candidate = current.join("data/generated").join(sub_path);

        if candidate.exists() {
            return Some(candidate);
        }

        if !current.pop() {
            break;
        }
    }

    Some(Path::new("data/generated").join(sub_path))
}

pub fn load_registry_manager(
    protocol_version: ProtocolVersion,
    registries: &[RegistryKeys],
) -> crate::Result<RegistryManager> {
    crate::Error::incompatible_version(
        protocol_version,
        ProtocolVersion::V1_16,
        ProtocolVersion::latest(),
    )?;
    let path = find_data_path(&protocol_version.data().to_string())
        .ok_or(crate::Error::DataPathNotFound)?;
    let resource_root = Path::new(&path);
    Ok(RegistryManager::builder()
        .register_all(registries)
        .load_from_resource_path(resource_root))
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

pub fn get_dimension<'a>(
    registry_manager: &'a RegistryManager,
    dimension_identifier: &Identifier,
) -> crate::Result<&'a RegistryEntry> {
    registry_manager
        .try_get(&RegistryKeys::DimensionType)
        .and_then(|reg| reg.try_get(dimension_identifier))
        .ok_or_else(|| crate::Error::UnknownRegistryEntry(dimension_identifier.clone()))
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
