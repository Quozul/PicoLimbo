use crate::blocks_report_loader::{BlockState, BlocksReport};
use crate::light::{get_emitted_light_level, is_transparent};
use blocks_report_data::internal_mapping::{
    InternalBlockMapping, InternalId, InternalMapping, InternalProperties, InternalState, StateData,
};
use minecraft_protocol::prelude::LengthPaddedVec;
use std::collections::{HashMap, HashSet};

pub fn build_internal_id_mapping(blocks_reports: &[BlocksReport]) -> InternalMapping {
    let mut state_registry: HashMap<(String, LengthPaddedVec<InternalProperties>), InternalId> =
        HashMap::new();
    let mut default_state_properties: HashMap<String, LengthPaddedVec<InternalProperties>> =
        HashMap::new();
    let mut next_internal_id: InternalId = 0;

    // 1. Discover all unique states and assign them a unique internal ID
    for report in blocks_reports {
        for (name, block) in &report.block_data.blocks {
            for state in &block.states {
                let properties = sort_internal_properties(state);
                let state_key = (name.clone(), properties.clone());

                state_registry.entry(state_key).or_insert_with(|| {
                    let new_id = next_internal_id;
                    next_internal_id += 1;
                    new_id
                });

                if state.default {
                    default_state_properties.insert(name.clone(), properties);
                }
            }
        }
    }

    // 2. Group the registered states by block name and build the final mappings
    let mut grouped_states: HashMap<String, Vec<InternalState>> = HashMap::new();

    for ((name, properties), internal_id) in state_registry {
        let is_transparent = is_transparent(&name);
        let get_light_level = get_emitted_light_level(&name);
        grouped_states.entry(name).or_default().push(InternalState {
            state_data: StateData::new(internal_id, is_transparent, get_light_level),
            properties,
        });
    }

    let mapping_set = grouped_states
        .into_iter()
        .map(|(name, mut states)| {
            let is_transparent = is_transparent(&name);
            let get_light_level = get_emitted_light_level(&name);

            let default_props = default_state_properties
                .get(&name)
                .expect("Default state properties not found for block");

            let default_internal_id = states
                .iter()
                .find(|s| &s.properties == default_props)
                .map(|s| s.state_data.internal_id())
                .expect("Could not find internal ID for the default state");

            states.sort_by_key(|s| s.state_data.internal_id());

            InternalBlockMapping {
                name,
                states: LengthPaddedVec::new(states),
                default_state_data: StateData::new(
                    default_internal_id,
                    is_transparent,
                    get_light_level,
                ),
            }
        })
        .collect::<HashSet<InternalBlockMapping>>();

    let mapping = mapping_set
        .into_iter()
        .collect::<Vec<InternalBlockMapping>>();
    InternalMapping {
        mapping: LengthPaddedVec::new(mapping),
    }
}

pub fn sort_internal_properties(state: &BlockState) -> LengthPaddedVec<InternalProperties> {
    let mut properties: Vec<InternalProperties> = state
        .properties
        .as_ref()
        .map(|props| {
            props
                .iter()
                .map(|(p_name, p_value)| InternalProperties {
                    name: p_name.clone(),
                    value: p_value.clone(),
                })
                .collect()
        })
        .unwrap_or_default();
    properties.sort();
    LengthPaddedVec::new(properties)
}
