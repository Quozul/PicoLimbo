use crate::block_state::{BlockState, BlocksReport};
use minecraft_protocol::prelude::LengthPaddedVec;
use minecraft_protocol::prelude::*;
use std::collections::{HashMap, HashSet};
use tracing::warn;

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
        grouped_states.entry(name).or_default().push(InternalState {
            internal_id,
            properties,
        });
    }

    let mapping_set = grouped_states
        .into_iter()
        .map(|(name, mut states)| {
            let default_internal_id =
                default_state_properties
                    .get(&name)
                    .and_then(|default_props| {
                        states
                            .iter()
                            .find(|s| &s.properties == default_props)
                            .map(|s| s.internal_id)
                            .or_else(|| states.first().map(|s| s.internal_id))
                    });

            if default_internal_id.is_none() {
                warn!("Could not find internal ID for the default state of '{name}'",);
            }

            let default_internal_id = default_internal_id.unwrap_or(0);

            states.sort_by_key(|s| s.internal_id);
            InternalBlockMapping {
                name,
                states: LengthPaddedVec::new(states),
                default_internal_id,
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

#[derive(Eq, PartialEq, Clone, PacketOut, PacketIn, Hash, Ord, PartialOrd)]
pub struct InternalProperties {
    pub name: String,
    pub value: String,
}

#[derive(Eq, PartialEq, PacketOut, PacketIn, Hash)]
pub struct InternalState {
    pub internal_id: InternalId,
    pub properties: LengthPaddedVec<InternalProperties>,
}

#[derive(Eq, PartialEq, PacketOut, PacketIn, Hash)]
pub struct InternalBlockMapping {
    pub name: String,
    pub states: LengthPaddedVec<InternalState>,
    pub default_internal_id: InternalId,
}

#[derive(PacketOut, PacketIn)]
pub struct InternalMapping {
    pub mapping: LengthPaddedVec<InternalBlockMapping>,
}

pub type InternalId = u32;
