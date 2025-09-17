pub mod blocks_report_loader;
pub mod build_report_mappings;
pub mod internal_mapping;

use crate::blocks_report_loader::{BlocksReport, load_block_data};
use crate::build_report_mappings::build_report_mappings;
use crate::internal_mapping::build_internal_id_mapping;
use minecraft_protocol::prelude::{BinaryWriter, EncodePacket};
use proc_macro2::{Ident, Span};
use protocol_version::protocol_version::ProtocolVersion;
use quote::quote;
use std::path::Path;
use std::{env, fs};

fn main() -> anyhow::Result<()> {
    let out_dir = env::var("OUT_DIR")?;
    let out_path = Path::new(&out_dir);

    // 1. Load all blocks reports
    let blocks_reports: Vec<BlocksReport> = load_block_data()?;

    // 2. Create internal mapping
    let internal_mapping = build_internal_id_mapping(&blocks_reports);

    // 3. Serialize internal mapping
    let save_path = out_path.join("internal_mapping");
    write(&internal_mapping, &save_path)?;

    // 4. Create report mappings
    let mut mappings_arms = Vec::new();
    let report_mappings = build_report_mappings(&blocks_reports, &internal_mapping);
    for mapping in report_mappings {
        let file_name = format!("version_mapping_{}", mapping.protocol_version);
        let save_path = out_path.join(file_name);
        write(&mapping.mapping, &save_path)?;

        let version_ident = Ident::new(&mapping.protocol_version.to_string(), Span::call_site());
        let file_path_str = save_path.to_str().unwrap().to_string();
        let arm = quote! {
            ProtocolVersion::#version_ident => {
                let bytes = include_bytes!(#file_path_str);
                let mut reader = minecraft_protocol::prelude::BinaryReader::new(bytes);
                Ok(ReportIdMapping::decode(&mut reader, minecraft_protocol::prelude::ProtocolVersion::latest())?)
            },
        };

        mappings_arms.push(arm);
    }

    let generated_code = quote! {

        #[allow(clippy::match_same_arms)]
        pub fn get_blocks_reports(protocol_version: minecraft_protocol::prelude::ProtocolVersion) -> Result<ReportIdMapping, BlockReportIdMappingError> {
            match protocol_version {
                #(#mappings_arms)*
                _ => Err(BlockReportIdMappingError::UnsupportedVersion(protocol_version)),
            }
        }
    };

    let dest_path = out_path.join("get_blocks_reports.rs");
    fs::write(&dest_path, generated_code.to_string())?;

    Ok(())
}

fn write<T: EncodePacket>(element: &T, save_path: &Path) -> anyhow::Result<()> {
    let mut writer = BinaryWriter::new();
    element.encode(&mut writer, ProtocolVersion::latest())?;
    let bytes = writer.into_inner();
    fs::write(save_path, bytes)?;
    Ok(())
}
