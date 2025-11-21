use minecraft_protocol::prelude::*;
use registries_data::grouped_registries::{
    V1_20_5Registries, V1_20_5RegistryEntries, V1_20_5RegistryEntry,
};
use serde_json::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use walkdir::WalkDir;

const AVAILABLE_REGISTRIES: [&str; 16] = [
    "minecraft:banner_pattern",
    "minecraft:chat_type",
    "minecraft:damage_type",
    "minecraft:dimension_type",
    "minecraft:painting_variant",
    "minecraft:trim_material",
    "minecraft:trim_pattern",
    "minecraft:wolf_variant",
    "minecraft:worldgen/biome",
    // The following were added in 1.21.5
    "minecraft:cat_variant",
    "minecraft:chicken_variant",
    "minecraft:cow_variant",
    "minecraft:frog_variant",
    "minecraft:pig_variant",
    "minecraft:wolf_sound_variant",
    // The following were added in 1.21.11
    "minecraft:zombie_nautilus_variant",
];

const REGISTRIES_TO_SEND: [&str; 17] = [
    "banner_pattern",
    "chat_type",
    "damage_type",
    "dimension_type",
    "painting_variant",
    "trim_material",
    "trim_pattern",
    "wolf_variant",
    "worldgen/biome",
    "worldgen\\biome",
    // The following were added in 1.21.5
    "cat_variant",
    "chicken_variant",
    "cow_variant",
    "frog_variant",
    "pig_variant",
    "wolf_sound_variant",
    "zombie_nautilus_variant",
];

pub fn get_version_directory(protocol_version: ProtocolVersion, data_location: &Path) -> PathBuf {
    data_location
        .join(protocol_version.data().to_string())
        .join("data")
        .join("minecraft")
}

pub fn encode<T>(encodable: &T, protocol_version: ProtocolVersion) -> Vec<u8>
where
    T: EncodePacket,
{
    let mut writer = BinaryWriter::new();
    encodable.encode(&mut writer, protocol_version).unwrap();
    writer.into_inner()
}

/// Way to get registries since 1.20.5
pub fn get_v1_20_5_registries(
    protocol_version: ProtocolVersion,
    data_location: &Path,
) -> V1_20_5Registries {
    let source_map = get_grouped_registries(protocol_version, data_location);
    let converted_registries: Vec<V1_20_5RegistryEntries> = source_map
        .iter()
        .map(|(registry_id, entries_list)| {
            let converted_entries: Vec<V1_20_5RegistryEntry> = entries_list
                .iter()
                .map(|(entry_id, entry_nbt)| V1_20_5RegistryEntry {
                    entry_id: entry_id.clone(),
                    nbt_bytes: LengthPaddedVec::new(encode(entry_nbt, protocol_version)),
                })
                .collect();

            V1_20_5RegistryEntries {
                registry_id: registry_id.clone(),
                entries: LengthPaddedVec::new(converted_entries),
            }
        })
        .collect();

    V1_20_5Registries {
        registries: LengthPaddedVec::new(converted_registries),
    }
}

fn get_grouped_registries(
    protocol_version: ProtocolVersion,
    data_location: &Path,
) -> HashMap<Identifier, Vec<(Identifier, Nbt)>> {
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
                let registry_str =
                    format!("minecraft:{}", parent_dir.to_string_lossy()).replace("\\", "/");
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
                (entry_id, nbt)
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
    let grouped = get_grouped_registries(protocol_version, data_location)
        .iter()
        .map(|(registry_id, entries)| {
            let value = entries
                .iter()
                .enumerate()
                .map(|(id, entry)| {
                    Nbt::nameless_compound(vec![
                        Nbt::string("name", &entry.0),
                        Nbt::int("id", id as i32),
                        entry.1.clone().set_name("element".to_string()),
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
