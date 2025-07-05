use pico_codegen::prelude::{BinaryReader, StringIndexer};
use std::collections::HashMap;

type InternalStringId = u16;
type PropertyName = InternalStringId;
type PropertyValue = InternalStringId;
pub type Property = (PropertyName, PropertyValue);
type StateId = u32;
pub type BlockId = InternalStringId;
type ProtocolVersionNumber = u16;

#[derive(Clone, Default)]
struct BlockState {
    state_id: StateId,
    properties: Vec<Property>,
}

#[derive(Clone, Default)]
pub struct BlocksReport {
    default_id: StateId,
    states: Vec<BlockState>,
}

impl BlocksReport {
    pub fn find_matching_state_id(
        &self,
        expected_properties: Vec<(PropertyName, PropertyValue)>,
    ) -> Option<StateId> {
        for state in &self.states {
            if state.properties == expected_properties {
                return Some(state.state_id);
            }
        }
        None
    }

    pub fn get_default_id(&self) -> StateId {
        self.default_id
    }
}

#[derive(Clone, Default)]
pub struct BlocksReports {
    string_indexer: StringIndexer,
    versions: HashMap<ProtocolVersionNumber, HashMap<BlockId, BlocksReport>>,
}

impl BlocksReports {
    pub fn new() -> Option<Self> {
        Some(Self {
            string_indexer: get_string_indexer()?,
            versions: read_all_versions(),
        })
    }

    pub fn get_internal_id(&self, string: &str) -> Option<InternalStringId> {
        self.string_indexer.get_index(string)
    }

    pub fn get_version(
        &self,
        version_number: ProtocolVersionNumber,
        block_id: BlockId,
    ) -> Option<&BlocksReport> {
        self.versions.get(&version_number)?.get(&block_id)
    }
}

static DATA: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/blocks.bin"));

fn read_all_versions() -> HashMap<ProtocolVersionNumber, HashMap<BlockId, BlocksReport>> {
    let mut reader = BinaryReader::new(DATA);
    let version_count = reader.read_u16();
    let mut versions = HashMap::with_capacity(version_count as usize);
    for _ in 0..version_count {
        let version_number = reader.read_u16();
        let index = reader.read_usize();
        let version = read_version(index);
        versions.insert(version_number, version);
    }
    versions
}

fn read_version(index: usize) -> HashMap<BlockId, BlocksReport> {
    let mut reader = BinaryReader::new(&DATA[index..]);
    let num_blocks = reader.read_u16();
    let mut blocks = HashMap::with_capacity(num_blocks as usize);

    for _ in 0..num_blocks {
        let block_id = reader.read_u16();
        let default_id = reader.read_u32();
        let property_count = reader.read_u16();
        let state_count = reader.read_u16();

        let mut states = Vec::with_capacity(state_count as usize);

        for _ in 0..state_count {
            let mut properties = Vec::new();

            for _ in 0..property_count {
                let property_name = reader.read_u16();
                let property_value = reader.read_u16();
                properties.push((property_name, property_value));
            }

            let state_id = reader.read_u32();

            properties.sort();
            states.push(BlockState {
                properties,
                state_id,
            });
        }

        let block = BlocksReport { states, default_id };
        blocks.insert(block_id, block);
    }
    blocks
}

fn get_string_indexer() -> Option<StringIndexer> {
    let mut reader = BinaryReader::new(DATA);

    // Skip version header
    let version_count = reader.read_u16();
    let version_size = size_of::<u16>() /*pvn*/ + size_of::<usize>() /*index*/;
    let header_size = version_count as usize * version_size;
    let header_size = size_of::<u16>() /*size of version count*/ + header_size;

    StringIndexer::from_bytes(&DATA[header_size..]).ok()
}
