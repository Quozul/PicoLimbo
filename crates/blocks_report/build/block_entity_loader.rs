use protocol_version::protocol_version::ProtocolVersion;
use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::fs::{self, DirEntry};
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Deserialize)]
struct Registries {
    #[serde(rename = "minecraft:block_entity_type")]
    block_entity_type: BlockEntityTypeRegistry,
}

#[derive(Deserialize)]
struct BlockEntityTypeRegistry {
    entries: HashMap<String, Entry>,
}

#[derive(Deserialize)]
struct Entry {
    protocol_id: i32,
}

#[derive(Clone)]
pub struct BlockEntityReport {
    pub protocol_version: ProtocolVersion,
    pub type_map: HashMap<String, i32>,
}

pub fn load_block_entity_data() -> anyhow::Result<Vec<BlockEntityReport>> {
    let data_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?)
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("data")
        .join("generated");

    let mut reports: Vec<BlockEntityReport> = fs::read_dir(data_dir)?
        .filter_map(|result| result.ok())
        .filter_map(|entry: DirEntry| {
            let name = entry.file_name().to_string_lossy().into_owned();
            ProtocolVersion::from_str(&name)
                .ok()
                .and_then(|protocol_version| {
                    if protocol_version.is_after_inclusive(ProtocolVersion::V1_16) {
                        let version_path = entry.path();
                        let registries_path = version_path.join("reports").join("registries.json");

                        fs::read_to_string(&registries_path)
                            .ok()
                            .and_then(|json_str| {
                                serde_json::from_str::<Registries>(&json_str).ok().map(
                                    |registries| {
                                        let type_map = registries
                                            .block_entity_type
                                            .entries
                                            .into_iter()
                                            .map(|(name, entry)| (name, entry.protocol_id))
                                            .collect();

                                        BlockEntityReport {
                                            protocol_version,
                                            type_map,
                                        }
                                    },
                                )
                            })
                    } else {
                        None
                    }
                })
        })
        .collect();

    reports.sort_by(|a, b| a.protocol_version.cmp(&b.protocol_version));
    Ok(reports)
}
