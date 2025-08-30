use blocks_report_data::internal_mapping::{InternalId, InternalMapping};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BlockStateBuilderError {
    #[error("Mismatched brackets: missing ']'")]
    MismatchedBrackets,
    #[error("Invalid property format: '{0}'")]
    InvalidPropertyFormat(String),
}

pub struct BlockStateBuilder<'a> {
    mapping: &'a InternalMapping,
    block_name: Option<String>,
    properties: HashMap<String, String>,
}

impl<'a> BlockStateBuilder<'a> {
    pub fn new(mapping: &'a InternalMapping) -> Self {
        Self {
            mapping,
            block_name: None,
            properties: HashMap::new(),
        }
    }

    pub fn with_state_string(
        &mut self,
        state_str: &str,
    ) -> Result<&mut Self, BlockStateBuilderError> {
        self.block_name = None;
        self.properties.clear();

        if let Some((name, props_part)) = state_str.split_once('[') {
            self.block_name = Some(name.to_string());

            let props_inner = props_part
                .strip_suffix(']')
                .ok_or(BlockStateBuilderError::MismatchedBrackets)?;

            if !props_inner.is_empty() {
                for pair in props_inner.split(',') {
                    let (key, value) = pair.split_once('=').ok_or_else(|| {
                        BlockStateBuilderError::InvalidPropertyFormat(pair.to_string())
                    })?;
                    self.properties
                        .insert(key.trim().to_string(), value.trim().to_string());
                }
            }
        } else {
            self.block_name = Some(state_str.to_string());
        }

        Ok(self)
    }

    pub fn build(&self) -> Option<InternalId> {
        let block_name = self.block_name.as_ref()?;
        let block_mapping = self
            .mapping
            .mapping
            .inner()
            .iter()
            .find(|b| &b.name == block_name)?;

        if self.properties.is_empty() {
            return Some(block_mapping.default_internal_id);
        }

        'state_loop: for state in block_mapping.states.inner().iter() {
            if state.properties.inner().len() != self.properties.len() {
                continue;
            }

            for (key, value) in &self.properties {
                let has_matching_prop = state
                    .properties
                    .inner()
                    .iter()
                    .any(|p| &p.name == key && &p.value == value);

                if !has_matching_prop {
                    continue 'state_loop;
                }
            }

            return Some(state.internal_id);
        }

        None
    }
}
