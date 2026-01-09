use crate::internal_mapping::{InternalBlockMapping, InternalMapping, StateData};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BlockStateLookupError {
    #[error("Mismatched brackets: missing ']'")]
    MismatchedBrackets,
    #[error("Invalid property format: '{0}'")]
    InvalidPropertyFormat(String),
    #[error("Unknown block: '{0}'")]
    UnknownBlock(String),
    #[error("No matching state found for block '{0}' with the given properties")]
    NoMatchingState(String),
}

pub struct BlockStateLookup<'a> {
    mapping: &'a InternalMapping,
}

impl<'a> BlockStateLookup<'a> {
    pub fn new(mapping: &'a InternalMapping) -> Self {
        Self { mapping }
    }

    /// Parses a block state string like "minecraft:chest[facing=north,type=single]"
    /// and returns the corresponding internal ID.
    pub fn parse_state_string(&self, state_str: &str) -> Result<&StateData, BlockStateLookupError> {
        // 1: Parse the raw string into a name and a map of property slices
        let (block_name, properties) = self.parse_name_and_properties(state_str)?;

        // 2: Find the corresponding block definition
        let block_mapping = self.find_block_mapping(block_name)?;

        // 3: Find the specific state based on the properties
        // If the properties map is empty, we get the default state.
        if properties.is_empty() {
            Ok(&block_mapping.default_state_data)
        } else {
            self.find_state_in_block(block_mapping, &properties)
                .ok_or_else(|| BlockStateLookupError::NoMatchingState(block_name.to_string()))
        }
    }

    /// Helper to parse "name[key1=val1,key2=val2]" into ("name", {"key1": "val1", "key2": "val2"}).
    /// Uses &str slices for efficiency, avoiding string allocations.
    fn parse_name_and_properties<'s>(
        &self,
        state_str: &'s str,
    ) -> Result<(&'s str, HashMap<&'s str, &'s str>), BlockStateLookupError> {
        if let Some((name, props_part)) = state_str.split_once('[') {
            let props_inner = props_part
                .strip_suffix(']')
                .ok_or(BlockStateLookupError::MismatchedBrackets)?;

            if props_inner.is_empty() {
                return Ok((name, HashMap::new()));
            }

            let mut properties = HashMap::new();
            for pair in props_inner.split(',') {
                let (key, value) = pair.split_once('=').ok_or_else(|| {
                    BlockStateLookupError::InvalidPropertyFormat(pair.to_string())
                })?;
                properties.insert(key.trim(), value.trim());
            }
            Ok((name, properties))
        } else {
            Ok((state_str, HashMap::new()))
        }
    }

    fn find_block_mapping(
        &self,
        block_name: &str,
    ) -> Result<&'a InternalBlockMapping, BlockStateLookupError> {
        self.mapping
            .mapping
            .inner()
            .iter()
            .find(|b| b.name == block_name)
            .ok_or_else(|| BlockStateLookupError::UnknownBlock(block_name.to_string()))
    }

    fn find_state_in_block(
        &self,
        block_mapping: &'a InternalBlockMapping,
        properties: &HashMap<&str, &str>,
    ) -> Option<&StateData> {
        block_mapping
            .states
            .inner()
            .iter()
            .find(|state| {
                if state.properties.inner().len() != properties.len() {
                    return false;
                }

                properties.iter().all(|(key, value)| {
                    state
                        .properties
                        .inner()
                        .iter()
                        .any(|p| p.name == *key && p.value == *value)
                })
            })
            .map(|state| &state.state_data)
    }
}
