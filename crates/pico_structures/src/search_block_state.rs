use minecraft_protocol::protocol_version::ProtocolVersion;
use pico_codegen::prelude::{BinaryReader, StringIndexer};

#[derive(Debug, Default)]
struct Property {
    name_id: u16,
    value_id: u16,
}

impl Property {
    fn new(name: String, value: String) -> Option<Self> {
        Some(Self {
            name_id: string_to_index(&name)?,
            value_id: string_to_index(&value)?,
        })
    }
}

#[derive(Debug, Default)]
struct BlockName(u16);

impl BlockName {
    fn new(name: String) -> Option<Self> {
        Some(Self(string_to_index(&name)?))
    }
}

#[derive(Debug, Default)]
pub struct SearchState {
    block_name: BlockName,
    version: ProtocolVersion,
    properties: Vec<Property>,
}

impl SearchState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn block_name(&mut self, name: impl ToString) -> &mut Self {
        self.block_name = BlockName::new(name.to_string()).unwrap_or_default();
        self
    }

    pub fn version(&mut self, version: ProtocolVersion) -> &mut Self {
        self.version = version;
        self
    }

    pub fn property(&mut self, name: impl ToString, value: impl ToString) -> &mut Self {
        if let Some(property) = Property::new(name.to_string(), value.to_string()) {
            self.properties.push(property);
        }
        self
    }

    pub fn find(&mut self) -> Option<i32> {
        let bytes = get_version_bytes(self.version.version_number() as u16).unwrap();
        let mut reader = BinaryReader::new(bytes);
        let num_blocks = reader.read_u16();

        for _i in 0..num_blocks {
            let block_name_id = reader.read_u16();

            if block_name_id == self.block_name.0 {
                let block_id = reader.read_u32();
                let state_id = self.find_property(&mut reader);
                return Some(state_id.unwrap_or(block_id as i32));
            } else {
                reader.skip::<u32>();
                let property_count = reader.read_u16() as usize;
                let state_count = reader.read_u16() as usize;
                let total_bytes = state_count * property_count * 4 + 4 * state_count;
                reader.skip_bytes(total_bytes);
            }
        }

        None
    }

    fn find_property(&self, reader: &mut BinaryReader) -> Option<i32> {
        let property_count = reader.read_u16();
        let state_count = reader.read_u16();
        let expected_properties = &self
            .properties
            .iter()
            .map(|p| (p.name_id, p.value_id))
            .collect::<Vec<_>>();

        for _ in 0..state_count {
            let mut state_properties = Vec::new();

            for _ in 0..property_count {
                let property_name = reader.read_u16();
                let property_value = reader.read_u16();
                state_properties.push((property_name, property_value));
            }

            let state_id = reader.read_u32();
            if compare_vecs(&state_properties, expected_properties) {
                return Some(state_id as i32);
            }
        }

        None
    }
}

static DATA: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/blocks.bin"));

fn get_version_bytes<'a>(version: u16) -> Option<&'a [u8]> {
    let mut reader = BinaryReader::new(DATA);
    let version_count = reader.read_u16();
    for _ in 0..version_count {
        let version_number = reader.read_u16();
        let index = reader.read_usize();
        if version_number == version {
            return Some(&DATA[index..]);
        }
    }
    None
}

fn string_to_index(name: &str) -> Option<u16> {
    let mut reader = BinaryReader::new(DATA);
    let version_count = reader.read_u16();
    let header_size = size_of::<u16>() /*version count*/ + version_count as usize * (size_of::<u16>() /*pvn*/ + size_of::<usize>()/*index*/);
    let indexer = StringIndexer::from_bytes(&DATA[header_size..]).ok()?;
    indexer.get_index(name)
}

fn compare_vecs(vec1: &[(u16, u16)], vec2: &[(u16, u16)]) -> bool {
    if vec1.len() != vec2.len() {
        return false;
    }

    let mut sorted_vec1 = vec1.to_vec();
    let mut sorted_vec2 = vec2.to_vec();

    sorted_vec1.sort();
    sorted_vec2.sort();

    sorted_vec1 == sorted_vec2
}
