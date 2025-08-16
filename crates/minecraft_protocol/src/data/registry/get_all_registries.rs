use crate::prelude::{EncodePacket, Identifier, Nbt};
use pico_binutils::prelude::BinaryWriter;
use protocol_version::protocol_version::ProtocolVersion;
use serde_json::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use walkdir::WalkDir;

fn get_entry_index(
    protocol_version: ProtocolVersion,
    data_location: &Path,
    subdirectory: &str,
    entry_name: &str,
    default: usize,
) -> usize {
    let directory = get_version_directory(protocol_version, data_location).join(subdirectory);

    if !directory.is_dir() {
        return 1;
    }

    directory.read_dir().map_or(default, |entries| {
        entries
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let path = entry.path();
                if path.is_file() {
                    path.file_stem()
                        .and_then(|stem| stem.to_str())
                        .map(|s| s.to_string())
                } else {
                    None
                }
            })
            .position(|entry| entry == entry_name)
            .unwrap_or(default)
    })
}

pub fn get_the_void_index(protocol_version: ProtocolVersion, data_location: &Path) -> usize {
    get_entry_index(
        protocol_version,
        data_location,
        "worldgen/biome",
        "the_void",
        1,
    )
}

pub fn get_dimension_type_index(
    protocol_version: ProtocolVersion,
    data_location: &Path,
    dimension_name: String,
) -> usize {
    get_entry_index(
        protocol_version,
        data_location,
        "dimension_type",
        &dimension_name,
        0,
    )
}

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

const REGISTRIES_TO_SEND: [&str; 16] = [
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
        .join("data")
        .join("minecraft")
}

/// This enum holds the encoded registries data for a given format
pub enum Registries {
    /// Starting 1.20.5 and onwards, registries are split into several packets
    V1_20_5 {
        registries: HashMap<Identifier, Vec<(Identifier, Vec<u8>)>>,
    },
    /// Starting 1.20.2 up to 1.20.5 (included) when configuration state was introduced
    /// all registries are sent in the same packet
    V1_20_2 { registry_codec: Vec<u8> },
    /// For versions between 1.19 and 1.20 (included), the dimension is removed, we only send the registry codec
    V1_19 { registry_codec: Vec<u8> },
    /// For versions between 1.16.2 and 1.18.2 (included), we must send the dimension codec separately
    V1_16_2 {
        registry_codec: Vec<u8>,
        dimension: Vec<u8>,
    },
    /// For 1.16 and 1.16.1, we only have to send the registry codec
    V1_16 { registry_codec: Vec<u8> },
    /// For versions older than 1.16, there are no registries
    None,
}

pub fn get_registries(
    protocol_version: ProtocolVersion,
    data_location: &Path,
    dimension_name: String,
) -> Registries {
    let encode = |nbt: &Nbt| -> Vec<u8> {
        let mut writer = BinaryWriter::new();
        nbt.encode(&mut writer, protocol_version).unwrap();
        writer.into_inner()
    };

    if protocol_version.is_after_inclusive(ProtocolVersion::V1_20_5) {
        let entries = get_v1_20_5_registries(protocol_version, data_location)
            .iter()
            .map(|(registry_id, entries)| {
                let entries = entries
                    .iter()
                    .map(|(entry_id, nbt)| (entry_id.clone(), encode(nbt)))
                    .collect();
                (registry_id.clone(), entries)
            })
            .collect();
        Registries::V1_20_5 {
            registries: entries,
        }
    } else if protocol_version.between_inclusive(ProtocolVersion::V1_20_2, ProtocolVersion::V1_20_3)
    {
        let nbt = get_v1_16_2_registry_codec(protocol_version, data_location);
        Registries::V1_20_2 {
            registry_codec: encode(&nbt),
        }
    } else if protocol_version.between_inclusive(ProtocolVersion::V1_19, ProtocolVersion::V1_20) {
        let registry_codec = get_v1_16_2_registry_codec(protocol_version, data_location);
        Registries::V1_19 {
            registry_codec: encode(&registry_codec),
        }
    } else if protocol_version.between_inclusive(ProtocolVersion::V1_16_2, ProtocolVersion::V1_18_2)
    {
        let registry_codec = get_v1_16_2_registry_codec(protocol_version, data_location);

        let dimension_types = registry_codec
            .find_tag("minecraft:dimension_type")
            .unwrap()
            .find_tag("value")
            .unwrap()
            .get_vec()
            .unwrap();

        let dimension = dimension_types
            .iter()
            .find(|element| {
                element
                    .find_tag("name".to_string())
                    .is_some_and(|name| match name {
                        Nbt::String { value, .. } => value == &dimension_name,
                        _ => false,
                    })
            })
            .cloned()
            .unwrap_or_else(|| dimension_types.first().cloned().unwrap_or(Nbt::End));

        let dimension = dimension.find_tag("element").unwrap().clone();

        Registries::V1_16_2 {
            registry_codec: encode(&registry_codec),
            dimension: encode(&dimension),
        }
    } else if protocol_version.between_inclusive(ProtocolVersion::V1_16, ProtocolVersion::V1_16_1) {
        let registry_codec = get_v1_16_registry_codec(data_location).unwrap();
        Registries::V1_16 {
            registry_codec: encode(&registry_codec),
        }
    } else {
        Registries::None
    }
}

/// Way to get registries since 1.20.5
fn get_v1_20_5_registries(
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
fn get_v1_16_2_registry_codec(protocol_version: ProtocolVersion, data_location: &Path) -> Nbt {
    let grouped = get_v1_20_5_registries(protocol_version, data_location)
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
fn get_v1_16_registry_codec(data_location: &Path) -> anyhow::Result<Nbt> {
    let path = get_version_directory(ProtocolVersion::V1_16, data_location).join("dimension.json");
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let json: Value = serde_json::from_reader(reader)?;
    Ok(Nbt::from_json(&json, None))
}
