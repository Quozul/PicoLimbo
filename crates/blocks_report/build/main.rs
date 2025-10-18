pub mod block_entity_loader;
pub mod blocks_report_loader;
pub mod build_report_mappings;
pub mod internal_mapping;

use crate::block_entity_loader::load_block_entity_data;
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

    // 5. Load block entity type data
    let block_entity_reports = load_block_entity_data()?;

    // 6. Generate block entity lookup code
    let mut entity_arms = Vec::new();
    for report in block_entity_reports {
        let version_ident = Ident::new(&report.protocol_version.to_string(), Span::call_site());

        // Create the map literal
        let entries: Vec<_> = report
            .type_map
            .iter()
            .map(|(name, id)| {
                quote! {
                    map.insert(#name.to_string(), #id);
                }
            })
            .collect();

        let arm = quote! {
            ProtocolVersion::#version_ident => {
                let mut map = std::collections::HashMap::new();
                #(#entries)*
                BlockEntityTypeLookup { type_map: map }
            },
        };

        entity_arms.push(arm);
    }

    let entity_lookup_code = quote! {
        use std::collections::HashMap;

        pub struct BlockEntityTypeLookup {
            type_map: HashMap<String, i32>,
        }

        impl BlockEntityTypeLookup {
            pub fn get_type_id(&self, block_entity_name: &str) -> Option<i32> {
                self.type_map.get(block_entity_name).copied()
            }
        }

        #[allow(clippy::match_same_arms)]
        pub fn get_block_entity_lookup(protocol_version: minecraft_protocol::prelude::ProtocolVersion) -> BlockEntityTypeLookup {
            use minecraft_protocol::prelude::ProtocolVersion;

            match protocol_version {
                #(#entity_arms)*
                _ => {
                    // Fallback to latest if version not found
                    BlockEntityTypeLookup { type_map: HashMap::new() }
                }
            }
        }
    };

    let entity_dest_path = out_path.join("block_entity_lookup.rs");
    fs::write(&entity_dest_path, entity_lookup_code.to_string())?;

    Ok(())
}

fn write<T: EncodePacket>(element: &T, save_path: &Path) -> anyhow::Result<()> {
    let mut writer = BinaryWriter::new();
    element.encode(&mut writer, ProtocolVersion::latest())?;
    let bytes = writer.into_inner();
    fs::write(save_path, bytes)?;
    Ok(())
}
