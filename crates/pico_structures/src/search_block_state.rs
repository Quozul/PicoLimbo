use crate::blocks_report::{BlockId, BlocksReports, Property};
use minecraft_protocol::prelude::ProtocolVersion;

fn new_property(blocks_reports: &BlocksReports, name: String, value: String) -> Option<Property> {
    let name_id = blocks_reports.get_internal_id(&name)?;
    let value_id = blocks_reports.get_internal_id(&value)?;
    Some((name_id, value_id))
}

fn new_block_id(blocks_reports: &BlocksReports, name: String) -> Option<BlockId> {
    blocks_reports.get_internal_id(&name)
}

#[derive(Default)]
pub struct SearchState {
    block_name: BlockId,
    version: ProtocolVersion,
    properties: Vec<Property>,
}

impl SearchState {
    pub fn new() -> Self {
        Self { ..Self::default() }
    }

    /// Parses a block state string to build a `SearchState`.
    ///
    /// The string can be a simple block name like `"minecraft:stone"` or include
    /// a state definition like `"minecraft:oak_wall_sign[facing=east,waterlogged=false]"`.
    ///
    /// # Arguments
    /// * `blocks_reports`: A reference to the `BlocksReports` needed to resolve names.
    /// * `input`: The string slice to parse.
    ///
    /// # Returns
    /// * `Some(SearchState)` if the parsing is successful.
    /// * `None` if the input string is malformed (e.g., an opening `[` without a closing `]`).
    pub fn from_string(blocks_reports: &BlocksReports, input: &str) -> Option<Self> {
        let mut state = SearchState::new();

        if let Some((name_part, props_part)) = input.split_once('[') {
            let props_inner = props_part.strip_suffix(']')?;

            state.block_name(blocks_reports, name_part);

            if !props_inner.is_empty() {
                for prop in props_inner.split(',') {
                    if let Some((key, value)) = prop.split_once('=') {
                        state.property(blocks_reports, key.trim(), value.trim());
                    } else {
                        return None;
                    }
                }
            }
        } else {
            state.block_name(blocks_reports, input);
        }

        Some(state)
    }

    pub fn block_name(&mut self, blocks_reports: &BlocksReports, name: impl ToString) -> &mut Self {
        self.block_name = new_block_id(blocks_reports, name.to_string()).unwrap_or_default();
        self
    }

    pub fn version(&mut self, version: ProtocolVersion) -> &mut Self {
        self.version = version;
        self
    }

    pub fn property(
        &mut self,
        blocks_reports: &BlocksReports,
        name: impl ToString,
        value: impl ToString,
    ) -> &mut Self {
        if let Some(property) = new_property(blocks_reports, name.to_string(), value.to_string()) {
            self.properties.push(property);
        }
        self
    }

    pub fn find(&mut self, blocks_reports: &BlocksReports) -> Option<u32> {
        let protocol_version_number = self.version.version_number() as u16;
        let mut expected_properties = self.properties.clone();
        expected_properties.sort();

        blocks_reports
            .get_version(protocol_version_number, self.block_name)
            .map(|block| block.find_matching_state_id(expected_properties))
    }
}
