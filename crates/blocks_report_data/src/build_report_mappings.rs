use crate::block_state::BlocksReport;
use crate::internal_mapping::{
    InternalId, InternalMapping, InternalProperties, sort_internal_properties,
};
use crate::report_mapping::{BlocksReportId, ReportIdMapping};
use minecraft_protocol::prelude::LengthPaddedVec;
use protocol_version::protocol_version::ProtocolVersion;
use std::collections::HashMap;
use tracing::warn;

pub struct ReportMapping {
    pub protocol_version: ProtocolVersion,
    pub mapping: ReportIdMapping,
}

pub fn build_report_mappings(
    blocks_reports: &[BlocksReport],
    internal_mapping: &InternalMapping,
) -> Vec<ReportMapping> {
    let mut state_lookup_map =
        HashMap::<(String, LengthPaddedVec<InternalProperties>), InternalId>::new();
    for mapping in internal_mapping.mapping.inner() {
        for state in mapping.states.inner() {
            let key = (mapping.name.clone(), state.properties.clone());
            state_lookup_map.insert(key, state.internal_id);
        }
    }

    const FALLBACK_STONE_ID: BlocksReportId = 1;

    let num_internal_states = state_lookup_map.len();

    let mut all_mappings = Vec::new();

    for report in blocks_reports {
        let mut report_vec: Vec<BlocksReportId> = vec![FALLBACK_STONE_ID; num_internal_states];

        for (name, block) in &report.block_data.blocks {
            for state in &block.states {
                let original_id = state.id;
                let properties = sort_internal_properties(state);
                let lookup_key = (name.clone(), properties);
                if let Some(internal_id) = state_lookup_map.get(&lookup_key) {
                    report_vec[*internal_id as usize] = original_id;
                } else {
                    warn!("State from report not found in canonical mapping for '{name}'");
                }
            }
        }

        let mut largest_block_id = 0;
        for id in &report_vec {
            if *id > largest_block_id {
                largest_block_id = *id;
            }
        }

        all_mappings.push(ReportMapping {
            protocol_version: report.protocol_version,
            mapping: ReportIdMapping {
                ids: LengthPaddedVec::new(report_vec),
                bits_per_entry: bits_needed(largest_block_id) as u8,
            },
        });
    }

    all_mappings
}

pub const fn bits_needed(n: BlocksReportId) -> u32 {
    if n == 0 { 1 } else { 64 - n.leading_zeros() }
}
