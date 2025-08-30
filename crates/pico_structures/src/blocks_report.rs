use pico_binutils::prelude::{BinaryReader, BinaryReaderError, StringIndexer};
use std::collections::HashMap;

type InternalStringId = u16;
type PropertyName = InternalStringId;
type PropertyValue = InternalStringId;
pub type Property = (PropertyName, PropertyValue);
type StateId = u32;
pub type BlockId = InternalStringId;
type ProtocolVersionNumber = u16;
type VersionReport = HashMap<BlockId, BlocksReport>;
type AllVersionReports = HashMap<ProtocolVersionNumber, VersionReport>;

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
    ) -> StateId {
        if self.states.is_empty() {
            return self.default_id;
        }

        for state in &self.states {
            if state.properties == expected_properties {
                return state.state_id;
            }
        }

        self.default_id
    }
}

#[derive(Clone, Default)]
pub struct BlocksReports {
    string_indexer: StringIndexer,
    versions: HashMap<ProtocolVersionNumber, HashMap<BlockId, BlocksReport>>,
}

impl BlocksReports {
    pub fn new() -> Result<Self, BinaryReaderError> {
        Ok(Self {
            string_indexer: get_string_indexer()?,
            versions: read_all_versions()?,
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

fn read_all_versions() -> Result<AllVersionReports, BinaryReaderError> {
    let mut reader = BinaryReader::new(DATA);
    let version_count = reader.read::<u16>()?;
    let mut versions = HashMap::with_capacity(version_count as usize);
    for _ in 0..version_count {
        let version_number = reader.read::<u16>()?;
        let index = reader.read::<usize>()?;
        let version = read_version(index)?;
        versions.insert(version_number, version);
    }
    Ok(versions)
}

fn read_version(index: usize) -> Result<VersionReport, BinaryReaderError> {
    let mut reader = BinaryReader::new(&DATA[index..]);
    let num_blocks = reader.read::<u16>()?;
    let mut blocks = HashMap::with_capacity(num_blocks as usize);

    for _ in 0..num_blocks {
        let block_id = reader.read::<u16>()?;
        let default_id = reader.read::<u32>()?;
        let state_count = reader.read::<u16>()?;

        let states = if state_count > 0 {
            let property_count = reader.read::<u16>()?;

            let mut states = Vec::with_capacity(state_count as usize);

            for _ in 0..state_count {
                let mut properties = Vec::new();

                for _ in 0..property_count {
                    let property_name = reader.read::<u16>()?;
                    let property_value = reader.read::<u16>()?;
                    properties.push((property_name, property_value));
                }

                let state_id = reader.read::<u32>()?;

                properties.sort();
                states.push(BlockState {
                    properties,
                    state_id,
                });
            }

            states
        } else {
            Vec::new()
        };

        let block = BlocksReport { states, default_id };
        blocks.insert(block_id, block);
    }
    Ok(blocks)
}

fn get_string_indexer() -> Result<StringIndexer, BinaryReaderError> {
    let mut reader = BinaryReader::new(DATA);

    // Skip version header
    let version_count = reader.read::<u16>()?;
    let version_size = size_of::<u16>() /*pvn*/ + size_of::<usize>() /*index*/;
    let header_size = version_count as usize * version_size;
    let header_size = size_of::<u16>() /*size of version count*/ + header_size;

    StringIndexer::from_bytes(&DATA[header_size..])
}
