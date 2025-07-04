use crate::blocks_report::BlocksReport;
use pico_codegen::prelude::StringIndexer;
use proc_macro2::TokenStream;
use quote::{TokenStreamExt, quote};
use rayon::prelude::*;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::{env, fs};

mod blocks_report;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let out_path = Path::new(&out_dir);
    let dest_path = Path::new(&out_dir).join("generated.rs");
    let mut generated_code = TokenStream::new();

    println!("cargo:rerun-if-changed=data/");

    let block_reports = load_all_reports();
    let all_strings = build_string_map(&block_reports);
    println!("Debug: {all_strings:?}");

    let indexer = StringIndexer::new(all_strings);
    generated_code.append_all(indexer.codegen());

    for (version, blocks_report) in &block_reports {
        let binary_report_path = out_path.join(version);
        let bytes = blocks_report.to_bytes(&indexer);
        fs::write(&binary_report_path, bytes).unwrap();
    }

    generated_code.append_all(codegen_version_bytes(
        block_reports.keys().cloned().collect(),
    ));

    fs::write(dest_path, generated_code.to_string()).unwrap();
}

fn codegen_version_bytes(versions: Vec<String>) -> TokenStream {
    let out_dir = env::var("OUT_DIR").unwrap();
    let out_path = Path::new(&out_dir);
    let match_arms = versions.iter().map(|version| {
        let binary_report_path = out_path.join(version).to_str().unwrap().to_string();
        quote! {
            #version => Some(include_bytes!(#binary_report_path)),
        }
    });

    quote! {
        fn get_version_bytes(s: &str) -> Option<&[u8]> {
            match s {
                #(#match_arms)*
                _ => None,
            }
        }
    }
}

type BlocksReportMap = HashMap<String, BlocksReport>;

fn build_string_map(blocks_report_map: &BlocksReportMap) -> HashSet<String> {
    blocks_report_map
        .values()
        .flat_map(|report| report.get_all_strings())
        .collect()
}

fn load_all_reports() -> BlocksReportMap {
    let data_dir = Path::new("data/");

    let read_dir = match fs::read_dir(data_dir) {
        Ok(read_dir) => read_dir,
        Err(_) => return HashMap::new(),
    };

    let entries: Vec<_> = read_dir.filter_map(|entry| entry.ok()).collect();

    let reports: HashMap<String, BlocksReport> = entries
        .into_par_iter()
        .filter_map(|dir_entry| {
            let file_name = dir_entry.file_name().to_string_lossy().into_owned();

            if !dir_entry.file_type().ok()?.is_dir() {
                return None;
            }

            let blocks_report_path = dir_entry.path().join("blocks.json");

            BlocksReport::from_path(blocks_report_path)
                .ok()
                .map(|blocks_report| (file_name, blocks_report))
        })
        .collect();

    reports
}
