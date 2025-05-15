use crate::data::registry::registry_entry::RegistryEntry;
use crate::prelude::{Identifier, Nbt};
use crate::protocol_version::ProtocolVersion;
use serde_json::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use walkdir::WalkDir;

const AVAILABLE_REGISTRIES: [&str; 15] = [
    "minecraft:banner_pattern",
    "minecraft:chat_type",
    "minecraft:damage_type",
    "minecraft:dimension_type",
    "minecraft:painting_variant",
    "minecraft:trim_material",
    "minecraft:trim_pattern",
    "minecraft:wolf_variant",
    "minecraft:worldgen/biome",
    // Added in 1.21.5
    "minecraft:cat_variant",
    "minecraft:chicken_variant",
    "minecraft:cow_variant",
    "minecraft:frog_variant",
    "minecraft:pig_variant",
    "minecraft:wolf_sound_variant",
];

const REGISTRIES_TO_SEND: [&str; 15] = [
    "banner_pattern",
    "chat_type",
    "damage_type",
    "dimension_type",
    "painting_variant",
    "trim_material",
    "trim_pattern",
    "wolf_variant",
    "worldgen/biome",
    // Added in 1.21.5
    "cat_variant",
    "chicken_variant",
    "cow_variant",
    "frog_variant",
    "pig_variant",
    "wolf_sound_variant",
];

fn get_version_directory(protocol_version: ProtocolVersion, data_location: &Path) -> PathBuf {
    data_location
        .join(protocol_version.data())
        .join("data/minecraft")
}

/// Way to get registries since 1.20.5
pub fn get_v1_20_5_registries(
    protocol_version: ProtocolVersion,
    data_location: &Path,
) -> HashMap<Identifier, Vec<RegistryEntry>> {
    let version_directory = get_version_directory(protocol_version, data_location);

    WalkDir::new(&version_directory)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            REGISTRIES_TO_SEND
                .iter()
                .any(|needle| e.path().to_string_lossy().contains(needle))
        })
        .filter_map(|e| {
            let path = e.path();
            if !path.is_file() || path.extension().and_then(|e| e.to_str()) != Some("json") {
                return None;
            }

            let registry_id = {
                let suffix = path.strip_prefix(&version_directory).ok()?;
                let parent_dir = suffix.parent()?;
                let registry_str = format!("minecraft:{}", parent_dir.to_string_lossy());
                if !AVAILABLE_REGISTRIES.contains(&registry_str.as_str()) {
                    return None;
                }
                Identifier::from_str(&registry_str).ok()?
            };

            let nbt = {
                let file = File::open(path).ok()?;
                let json: Value = serde_json::from_reader(BufReader::new(file)).ok()?;
                Nbt::from_json(&json, None)
            };

            let entry = {
                let stem = path.file_stem()?.to_str()?;
                let entry_id = Identifier::minecraft(stem);
                RegistryEntry {
                    entry_id,
                    has_data: true,
                    nbt: Some(nbt),
                }
            };

            Some((registry_id, entry))
        })
        .fold(HashMap::new(), |mut acc, (rid, entry)| {
            acc.entry(rid).or_default().push(entry);
            acc
        })
}

/// Way to get registries since 1.16.2 up until 1.20.3
pub fn get_v1_16_2_registry_codec(protocol_version: ProtocolVersion, data_location: &Path) -> Nbt {
    let grouped = get_v1_20_5_registries(protocol_version.clone(), data_location)
        .iter()
        .map(|(registry_id, entries)| {
            let value = entries
                .iter()
                .enumerate()
                .map(|(id, entry)| {
                    Nbt::nameless_compound(vec![
                        Nbt::string("name", &entry.entry_id),
                        Nbt::int("id", id as i32),
                        entry.nbt.clone().unwrap().set_name("element".to_string()),
                    ])
                })
                .collect::<Vec<Nbt>>();

            Nbt::compound(
                registry_id,
                vec![
                    Nbt::string("type", registry_id),
                    Nbt::compound_list("value", value),
                ],
            )
        })
        .collect::<Vec<Nbt>>();

    Nbt::nameless_compound(grouped)
}

/// Way to get registries for 1.16 and 1.16.1
pub fn get_v1_16_registry_codec(data_location: &Path) -> anyhow::Result<Nbt> {
    let path = get_version_directory(ProtocolVersion::V1_16, data_location).join("dimension.json");
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let json: Value = serde_json::from_reader(reader)?;
    Ok(Nbt::from_json(&json, None))
}
