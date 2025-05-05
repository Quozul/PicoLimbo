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

fn get_version_directory(protocol_version: ProtocolVersion) -> PathBuf {
    let data_dir = std::env::var("DATA_DIR").unwrap_or_else(|_| "./data/generated".to_string());
    PathBuf::from(data_dir)
        .join(protocol_version.data())
        .join("data/minecraft")
}

/// Way to get registries since 1.20.5
pub fn get_v1_20_5_registries(
    protocol_version: ProtocolVersion,
) -> HashMap<Identifier, Vec<RegistryEntry>> {
    let version_directory = get_version_directory(protocol_version);
    let registries = get_all_registries(&version_directory);

    let mut grouped: HashMap<Identifier, Vec<RegistryEntry>> = HashMap::new();

    for registry in &registries {
        let entry = RegistryEntry {
            entry_id: Identifier::minecraft(&registry.entry_id),
            has_data: true,
            nbt: Some(registry.nbt.clone()),
        };
        grouped
            .entry(Identifier::from_str(&registry.registry_id).unwrap())
            .or_default()
            .push(entry);
    }

    grouped
}

/// Way to get registries since 1.16.2
pub fn get_v1_16_2_registry_codec(protocol_version: ProtocolVersion) -> Nbt {
    let grouped = get_v1_20_5_registries(protocol_version.clone());
    Nbt::Compound {
        name: None,
        value: grouped
            .iter()
            .map(|(registry_id, entries)| {
                let mut id = 0;
                Nbt::Compound {
                    name: Some(registry_id.to_string()),
                    value: vec![
                        Nbt::String {
                            name: Some(String::from("type")),
                            value: registry_id.to_string(),
                        },
                        Nbt::List {
                            name: Some(String::from("value")),
                            value: entries
                                .iter()
                                .map(|e| {
                                    let n = Nbt::Compound {
                                        name: None,
                                        value: vec![
                                            Nbt::String {
                                                name: Some("name".to_string()),
                                                value: e.entry_id.to_string(),
                                            },
                                            Nbt::Int {
                                                name: Some("id".to_string()),
                                                value: id,
                                            },
                                            e.nbt.clone().unwrap().set_name("element".to_string()),
                                        ],
                                    };
                                    id = id + 1;
                                    n
                                })
                                .collect(),
                            tag_type: 10,
                        },
                    ],
                }
            })
            .collect::<Vec<_>>(),
    }
}

/// Way to get registries for 1.16 and 1.16.1
pub fn get_v1_16_registry_codec() -> anyhow::Result<Nbt> {
    let path = get_version_directory(ProtocolVersion::V1_16).join("dimension.json");
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let json: Value = serde_json::from_reader(reader)?;
    Ok(Nbt::from_json(&json, None))
}

fn get_all_registries(root_directory: &Path) -> Vec<DataRegistryEntry> {
    WalkDir::new(root_directory)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            let path = e.path().to_str().unwrap_or_default();
            REGISTRIES_TO_SEND.iter().any(|s| path.contains(s))
        })
        .flat_map(|entry| json_to_nbt(root_directory, entry.path()))
        .filter(|entry| AVAILABLE_REGISTRIES.contains(&entry.registry_id.as_str()))
        .collect::<Vec<_>>()
}

#[derive(Debug)]
pub struct DataRegistryEntry {
    pub registry_id: String,
    pub entry_id: String,
    pub nbt: Nbt,
}

fn json_to_nbt(
    root_directory: &Path,
    path: &Path,
) -> Result<DataRegistryEntry, Box<dyn std::error::Error>> {
    if !path.is_file() || path.extension().and_then(|s| s.to_str()) != Some("json") {
        return Err(format!("{:?} not a json file", path).into());
    }

    let registry_id = get_registry_id(root_directory, path)?;
    let entry_id = get_entry_id(path)?;
    let json = get_json_files(path)?;
    let nbt = Nbt::from_json(&json, None);

    Ok(DataRegistryEntry {
        registry_id,
        entry_id,
        nbt,
    })
}

fn get_registry_id(
    root_directory: &Path,
    path: &Path,
) -> Result<String, Box<dyn std::error::Error>> {
    let suffix = path.strip_prefix(root_directory)?;
    Ok(format!(
        "minecraft:{}",
        suffix
            .parent()
            .ok_or("failed to read suffix")?
            .to_string_lossy()
    ))
}

fn get_entry_id(path: &Path) -> Result<String, &str> {
    path.file_stem()
        .and_then(|s| s.to_str())
        .map(|s| s.to_string())
        .ok_or("Failed to get file stem")
}

fn get_json_files(path: &Path) -> std::io::Result<Value> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    Ok(serde_json::from_reader(reader)?)
}
