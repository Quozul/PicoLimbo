use blocks_report_data::block_state::BlocksReport;
use protocol_version::protocol_version::ProtocolVersion;
use std::fs::DirEntry;
use std::path::PathBuf;
use std::str::FromStr;
use std::{env, fs};

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
                        println!("cargo:rerun-if-changed={}", blocks_report_path.display());
                        BlocksReport::from_path(&blocks_report_path, protocol_version).ok()
                    } else {
                        None
                    }
                })
        })
        .collect();

    block_data_list.sort_by(|a, b| a.protocol_version.cmp(&b.protocol_version));
    Ok(block_data_list)
}
