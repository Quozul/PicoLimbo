use crate::sanitize_version;
use pico_nbt::{IndexMap, NbtOptions, Value, from_slice_with_options, to_writer_with_options};
use pico_registries::registry_provider::{RegistryProvider, RuntimeRegistryProvider};
use protocol_version::protocol_version::ProtocolVersion;
use serde::Serialize;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use tracing::{debug, error, info, warn};

/// Represents the data associated with a specific registry type.
#[derive(Serialize)]
struct Registry {
    #[serde(rename = "type")]
    registry_type: String,
    value: Vec<RegistryEntry>,
}

/// A generic registry entry.
#[derive(Serialize)]
struct RegistryEntry {
    name: String,
    id: i32,
    element: Value,
}

#[derive(Serialize)]
struct Tag {
    #[serde(rename = "id")]
    registry_id: String,
    value: Vec<TagEntry>,
}

#[derive(Serialize)]
struct TagEntry {
    identifier: String,
    ids: Vec<i32>,
}

pub fn convert_data(version: &str) -> anyhow::Result<()> {
    let protocol_version = {
        let sanitized_version = sanitize_version(version);
        ProtocolVersion::from_str(&sanitized_version)?
    };

    let base_path = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR")?)
        .join("data")
        .join(protocol_version.to_string());
    move_reports(&base_path, protocol_version)?;

    if !protocol_version.has_registries() {
        warn!("{} does not have registries", version);
        return Ok(());
    }

    let registry_provider = load_registry_provider(protocol_version)?;
    convert_registries(&base_path, &registry_provider)?;
    convert_tags(&base_path, &registry_provider)?;

    Ok(())
}

fn convert_registries(
    base_path: &Path,
    registry_provider: &RuntimeRegistryProvider,
) -> anyhow::Result<()> {
    let output_nbt = base_path.join("registries.nbt");

    if output_nbt.exists() {
        info!(
            "Registries already generated, if you want to re-convert them, delete the file at: {}",
            output_nbt.display()
        );
        return Ok(());
    }

    if let Some(parent) = output_nbt.parent()
        && !parent.exists()
    {
        std::fs::create_dir_all(parent)?;
    }

    let data = registry_provider.get_registry_data_v1_20_5()?;
    let mut registries = IndexMap::new();
    for (identifier, entries) in data {
        let registry_value = entries
            .iter()
            .enumerate()
            .filter_map(|(id, entry)| {
                match from_slice_with_options(
                    &entry.nbt_bytes,
                    NbtOptions::new().nameless_root(true),
                ) {
                    Ok((_name, element)) => Some(RegistryEntry {
                        name: entry.entry_id.to_string(),
                        id: i32::try_from(id).expect("Failed to convert registry ID to i32"),
                        element,
                    }),
                    Err(err) => {
                        error!(?err, "Failed to parse NBT bytes");
                        debug!(?entry.nbt_bytes);
                        None
                    }
                }
            })
            .collect();

        registries.insert(
            identifier.to_string(),
            Registry {
                registry_type: identifier.to_string(),
                value: registry_value,
            },
        );
    }

    let options = NbtOptions::new().nameless_root(true);
    let mut file = File::create(output_nbt)?;
    to_writer_with_options(&mut file, &registries, None, options)?;
    Ok(())
}

fn convert_tags(
    base_path: &Path,
    registry_provider: &RuntimeRegistryProvider,
) -> anyhow::Result<()> {
    let output_nbt = base_path.join("tags.nbt");

    if output_nbt.exists() {
        info!(
            "Tag registries already generated, if you want to re-convert them, delete the file at: {}",
            output_nbt.display()
        );
        return Ok(());
    }

    if let Some(parent) = output_nbt.parent()
        && !parent.exists()
    {
        std::fs::create_dir_all(parent)?;
    }

    let data = registry_provider.get_tagged_registries()?;
    let mut tags = IndexMap::new();

    for tagged_registry in data {
        let tag_entries = tagged_registry
            .tags
            .iter()
            .map(|registry_tag| TagEntry {
                identifier: registry_tag.identifier.to_string(),
                ids: registry_tag
                    .ids
                    .iter()
                    .map(|id| i32::try_from(*id).unwrap())
                    .collect(),
            })
            .collect::<Vec<_>>();

        let identifier = tagged_registry.registry_id;
        tags.insert(
            identifier.to_string(),
            Tag {
                registry_id: identifier.to_string(),
                value: tag_entries,
            },
        );
    }

    let options = NbtOptions::new().nameless_root(true);
    let mut file = File::create(output_nbt)?;
    to_writer_with_options(&mut file, &tags, None, options)?;
    Ok(())
}

fn load_registry_provider(
    protocol_version: ProtocolVersion,
) -> anyhow::Result<RuntimeRegistryProvider> {
    let start_dir = PathBuf::new().join("cache").join("generated");
    Ok(RuntimeRegistryProvider::new_from_data_generator(
        &start_dir,
        protocol_version,
    )?)
}

fn move_reports(base_path: &Path, protocol_version: ProtocolVersion) -> anyhow::Result<()> {
    let start_dir = PathBuf::new()
        .join("cache")
        .join("generated")
        .join(protocol_version.packets().to_string())
        .join("reports");

    let copy = |from: &Path, to: &Path, file: &str| -> anyhow::Result<()> {
        let from = from.join(file);
        if !from.exists() {
            warn!("{} does not exists", file);
            return Ok(());
        }
        if !to.exists() {
            std::fs::create_dir_all(to)?;
        }
        let to = to.join(file);
        if to.exists() {
            info!(
                "{} already exists, if you want to re-convert them, delete the file at: {}",
                file,
                to.display()
            );
            return Ok(());
        }
        std::fs::copy(from, to)?;
        Ok(())
    };
    copy(&start_dir, base_path, "blocks.json")?;
    copy(&start_dir, base_path, "registries.json")?;
    copy(&start_dir, base_path, "packets.json")?;

    Ok(())
}
