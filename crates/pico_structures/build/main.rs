use crate::blocks_report::BlocksReport;
use minecraft_protocol::prelude::ProtocolVersion;
use pico_binutils::prelude::{BinaryWriter, BinaryWriterError, StringIndexer};
use rayon::prelude::*;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::{env, fs};

mod blocks_report;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("blocks.bin");

    let block_reports = load_all_reports();
    let all_strings = build_string_map(&block_reports);

    let indexer = StringIndexer::new(all_strings);

    let block_report_bytes = block_reports
        .par_iter()
        .filter_map(|(version, blocks_report)| {
            let protocol_version = ProtocolVersion::from_str(version).unwrap();
            if protocol_version.is_after_inclusive(ProtocolVersion::V1_19) {
                Some((
                    protocol_version,
                    blocks_report
                        .to_bytes(&indexer)
                        .expect("Failed to serialize block reports"),
                ))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    let indexer_bytes = indexer
        .to_bytes()
        .expect("Failed to serialize indexer reports");

    let mut header = block_reports_header(&block_report_bytes, indexer_bytes.len())
        .expect("Failed to serialize block reports");

    header.extend(indexer_bytes);

    for (_version, block_reports) in block_report_bytes {
        header.extend(block_reports);
    }

    fs::write(dest_path, header).expect("Unable to write file");
}

fn block_reports_header(
    block_reports: &Vec<(ProtocolVersion, Vec<u8>)>,
    indexer_offset: usize,
) -> Result<Vec<u8>, BinaryWriterError> {
    let mut writer = BinaryWriter::default();

    let version_count = block_reports.len() as u16;
    writer.write(&version_count)?;

    let mut index = indexer_offset + /*version_count size*/ size_of::<u16>() + version_count as usize * (size_of::<u16>() /*pvn*/ + size_of::<usize>()/*index*/);
    for (version, block_reports) in block_reports {
        writer.write(&(version.version_number() as u16))?;
        writer.write(&index)?;
        index += block_reports.len();
    }
    Ok(writer.into_inner())
}

type BlocksReportMap = HashMap<String, BlocksReport>;

fn build_string_map(blocks_report_map: &BlocksReportMap) -> HashSet<String> {
    blocks_report_map
        .values()
        .flat_map(|report| report.get_all_strings())
        .collect()
}

fn load_all_reports() -> BlocksReportMap {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let data_dir = manifest_dir
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("data")
        .join("generated");
    println!("cargo:rerun-if-changed={}", data_dir.display());

    let read_dir = match fs::read_dir(data_dir) {
        Ok(read_dir) => read_dir,
        Err(_) => return HashMap::new(),
    };

    let entries: Vec<_> = read_dir.filter_map(|entry| entry.ok()).collect();

    entries
        .into_par_iter()
        .filter_map(|dir_entry| {
            let file_name = dir_entry.file_name().to_string_lossy().into_owned();

            if !dir_entry.file_type().ok()?.is_dir() {
                return None;
            }

            let blocks_report_path = dir_entry.path().join("reports").join("blocks.json");

            BlocksReport::from_path(blocks_report_path)
                .ok()
                .map(|blocks_report| (file_name, blocks_report))
        })
        .collect()
}
