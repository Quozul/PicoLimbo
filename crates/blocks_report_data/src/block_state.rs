use crate::report_mapping::BlocksReportId;
use protocol_version::protocol_version::ProtocolVersion;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub struct BlocksReport {
    pub protocol_version: ProtocolVersion,
    pub block_data: BlockData,
}

impl BlocksReport {
    pub fn from_path<P: AsRef<Path>>(
        path: P,
        protocol_version: ProtocolVersion,
    ) -> std::io::Result<Self> {
        let block_data = BlockData::from_path(path)?;
        Ok(Self {
            protocol_version,
            block_data,
        })
    }
}

#[derive(Deserialize)]
pub struct BlockData {
    #[serde(flatten)]
    pub blocks: HashMap<String, Block>,
}

impl BlockData {
    pub fn from_path<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        let blocks_str = fs::read_to_string(&path)?;
        Ok(serde_json::from_str::<BlockData>(&blocks_str)?)
    }
}

#[derive(Deserialize)]
pub struct BlockDefinition {
    #[serde(alias = "type")]
    pub definition_type: String,
    pub properties: HashMap<String, String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Block {
    pub properties: Option<HashMap<String, Vec<String>>>,
    pub states: Vec<BlockState>,
    pub definition: Option<BlockDefinition>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BlockState {
    pub id: BlocksReportId,
    pub properties: Option<HashMap<String, String>>,
    #[serde(default)]
    pub default: bool,
}
