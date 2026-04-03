use blocks_report_data::report_mapping::BlocksReportId;
use protocol_version::protocol_version::ProtocolVersion;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::DirEntry;
use std::path::PathBuf;
use std::str::FromStr;
use std::{env, fs};

#[derive(Debug)]
pub struct BlocksReport {
    pub protocol_version: ProtocolVersion,
    pub block_data: BlockData,
}

#[derive(Debug, Deserialize)]
pub struct BlockData {
    #[serde(flatten)]
    pub blocks: HashMap<String, Block>,
}

#[derive(Debug, Deserialize)]
pub struct BlockDefinition {
    #[serde(alias = "type")]
    pub definition_type: String,
    pub properties: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Block {
    pub properties: Option<HashMap<String, Vec<String>>>,
    pub states: Vec<BlockState>,
    pub definition: Option<BlockDefinition>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BlockState {
    pub id: BlocksReportId,
    pub properties: Option<HashMap<String, String>>,
    #[serde(default)]
    pub default: bool,
}

pub fn load_block_data() -> anyhow::Result<Vec<BlocksReport>> {
    let data_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?)
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("data")
        .join("generated");

    let mut block_data_list: Vec<BlocksReport> = fs::read_dir(data_dir)?
        .filter_map(|result| result.ok())
        .filter_map(|entry: DirEntry| {
            let name = entry.file_name().to_string_lossy().into_owned();
            ProtocolVersion::from_str(&name)
                .ok()
                .and_then(|protocol_version| {
                    if protocol_version.is_after_inclusive(ProtocolVersion::V1_16) {
                        let version_path = entry.path();
                        let blocks_report_path = version_path.join("reports").join("blocks.json");
                        fs::read_to_string(&blocks_report_path)
                            .ok()
                            .and_then(|blocks_str| {
                                serde_json::from_str::<BlockData>(&blocks_str).ok().map(
                                    |block_data| BlocksReport {
                                        protocol_version,
                                        block_data,
                                    },
                                )
                            })
                    } else {
                        None
                    }
                })
        })
        .collect();

    block_data_list.sort_by(|a, b| a.protocol_version.cmp(&b.protocol_version));
    Ok(block_data_list)
}
