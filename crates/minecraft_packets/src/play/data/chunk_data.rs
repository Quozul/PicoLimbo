use crate::play::data::chunk_section::ChunkSection;
use crate::play::data::palette_container::PaletteContainer;
use minecraft_protocol::prelude::*;
use minecraft_protocol::protocol_version::ProtocolVersion;
use thiserror::Error;

#[derive(Debug)]
pub struct ChunkData {
    height_maps: Nbt,
    v1_21_5_height_maps: LengthPaddedVec<HeightMap>,
    /// Size of Data in bytes!
    /// LengthPaddedVec prefixes with the number of elements!
    data: Vec<ChunkSection>,
    block_entities: LengthPaddedVec<BlockEntity>,
}

#[derive(Debug, Error)]
enum HeightMapError {
    #[error(transparent)]
    Infallible(#[from] std::convert::Infallible),
    #[error(transparent)]
    Vec(#[from] LengthPaddedVecEncodeError),
}

#[derive(Debug)]
struct HeightMap {
    /// 1: WORLD_SURFACE
    /// All blocks other than air, cave air and void air. To determine if a beacon beam is obstructed.
    /// 4: MOTION_BLOCKING
    /// "Solid" blocks, except bamboo saplings and cactuses; fluids. To determine where to display rain and snow.
    /// 5: MOTION_BLOCKING_NO_LEAVES
    /// Same as MOTION_BLOCKING, excluding leaf blocks.
    height_map_type: VarInt,
    data: LengthPaddedVec<i64>,
}

impl EncodePacketField for HeightMap {
    type Error = HeightMapError;

    fn encode(&self, bytes: &mut Vec<u8>, protocol_version: u32) -> Result<(), Self::Error> {
        self.height_map_type.encode(bytes, protocol_version)?;
        self.data.encode(bytes, protocol_version)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct BlockEntity {
    // TODO: Implement BlockEntity
}

impl EncodePacketField for BlockEntity {
    type Error = std::convert::Infallible;

    fn encode(&self, _bytes: &mut Vec<u8>, _protocol_version: u32) -> Result<(), Self::Error> {
        // Nothing to encode
        Ok(())
    }
}

impl ChunkData {
    pub fn void(biome_index: i32) -> Self {
        let long_array_tag = Nbt::LongArray {
            name: Some("MOTION_BLOCKING".to_string()),
            value: vec![0; 37],
        };
        let root_tag = Nbt::Compound {
            name: None,
            value: vec![long_array_tag],
        };

        Self {
            height_maps: root_tag,
            v1_21_5_height_maps: LengthPaddedVec::new(vec![HeightMap {
                height_map_type: VarInt::new(4), // Motionblock type
                data: LengthPaddedVec::new(vec![0; 37]),
            }]),
            data: vec![ChunkSection::void(biome_index); 24],
            block_entities: Vec::new().into(),
        }
    }

    pub fn all_stone(void_biome_index: i32) -> Self {
        let long_array_tag = Nbt::LongArray {
            name: Some("MOTION_BLOCKING".to_string()),
            value: vec![0; 37],
        };
        let root_tag = Nbt::Compound {
            name: None,
            value: vec![long_array_tag],
        };

        let mut data = Vec::new();

        for i in 0..24 {
            let section = if i == 12 {
                let structure = SimpleHouse::new(1, 0, void_biome_index);
                ChunkSection::from_structure(structure)
            } else {
                ChunkSection::void(void_biome_index)
            };
            data.push(section);
        }

        Self {
            height_maps: root_tag,
            v1_21_5_height_maps: LengthPaddedVec::new(vec![HeightMap {
                height_map_type: VarInt::new(4), // Motionblock type
                data: LengthPaddedVec::new(vec![0; 37]),
            }]),
            data,
            block_entities: Vec::new().into(),
        }
    }
}

/// Trait for defining structures that can be converted to chunk sections
pub trait Structure {
    /// Get the block registry ID at the given coordinates within the structure
    /// Coordinates should be in the range 0-15 for a single chunk section
    fn get_block_at(&self, x: i32, y: i32, z: i32) -> i32;

    /// Get the biome registry ID at the given coordinates within the structure
    /// Coordinates should be in the range 0-15 for a single chunk section
    fn get_biome_at(&self, x: i32, y: i32, z: i32) -> i32;
}

impl ChunkSection {
    pub fn from_structure<S: Structure>(structure: S) -> Self {
        // Collect all unique blocks and biomes
        let mut block_palette = Vec::new();
        let mut biome_palette = Vec::new();

        // First pass: collect all unique block and biome IDs
        for y in 0..16 {
            for z in 0..16 {
                for x in 0..16 {
                    let block_id = structure.get_block_at(x, y, z);
                    if !block_palette.contains(&block_id) {
                        block_palette.push(block_id);
                    }
                }
            }
        }

        // For biomes, we only need to check every 4x4x4 region (64 total regions)
        for y in (0..16).step_by(4) {
            for z in (0..16).step_by(4) {
                for x in (0..16).step_by(4) {
                    let biome_id = structure.get_biome_at(x, y, z);
                    if !biome_palette.contains(&biome_id) {
                        biome_palette.push(biome_id);
                    }
                }
            }
        }

        // Determine bits per entry based on palette size
        let block_bits_per_entry = if block_palette.len() == 1 {
            0 // Single valued
        } else {
            std::cmp::max(4, (block_palette.len() as f64).log2().ceil() as u8)
        };

        let biome_bits_per_entry = if biome_palette.len() == 1 {
            0 // Single valued
        } else {
            std::cmp::max(1, (biome_palette.len() as f64).log2().ceil() as u8)
        };

        let mut block_count = 0i16;

        // Create block states container
        let block_states = if block_palette.len() == 1 {
            // Single valued palette
            if block_palette[0] != 0 {
                // Assuming 0 is air
                block_count = 4096;
            }
            PaletteContainer::SingleValued {
                bits_per_entry: 0,
                value: VarInt::new(block_palette[0]),
                data: Vec::new(),
            }
        } else {
            // Indirect palette
            let mut palette = Vec::new();
            for &block_id in &block_palette {
                palette.push(VarInt::new(block_id));
            }

            // Calculate data array size
            let total_bits = 4096 * block_bits_per_entry as usize;
            let data_size = total_bits.div_ceil(64); // Ceiling division by 64
            let mut data = vec![0i64; data_size];

            // Fill data array
            for y in 0..16 {
                for z in 0..16 {
                    for x in 0..16 {
                        let block_id = structure.get_block_at(x, y, z);
                        let palette_index =
                            block_palette.iter().position(|&id| id == block_id).unwrap();

                        if block_id != 0 {
                            // Assuming 0 is air
                            block_count += 1;
                        }

                        // Calculate position in data array
                        let block_number = ((y * 16) + z) * 16 + x;
                        let start_long = (block_number * block_bits_per_entry as i32) / 64;
                        let start_offset = (block_number * block_bits_per_entry as i32) % 64;

                        // Set the bits in the data array
                        if start_long < data.len() as i32 {
                            data[start_long as usize] |= (palette_index as i64) << start_offset;
                        }
                    }
                }
            }

            PaletteContainer::Indirect {
                bits_per_entry: block_bits_per_entry,
                palette: palette.into(),
                data,
            }
        };

        // Create biomes container
        let biomes = if biome_palette.len() == 1 {
            PaletteContainer::SingleValued {
                bits_per_entry: 0,
                value: VarInt::new(biome_palette[0]),
                data: Vec::new(),
            }
        } else {
            let mut palette = Vec::new();
            for &biome_id in &biome_palette {
                palette.push(VarInt::new(biome_id));
            }

            // For biomes, we have 64 entries (4x4x4 regions)
            let total_bits = 64 * biome_bits_per_entry as usize;
            let data_size = total_bits.div_ceil(64);
            let mut data = vec![0i64; data_size];

            // Fill biome data array
            let mut biome_index = 0;
            for y in (0..16).step_by(4) {
                for z in (0..16).step_by(4) {
                    for x in (0..16).step_by(4) {
                        let biome_id = structure.get_biome_at(x, y, z);
                        let palette_index =
                            biome_palette.iter().position(|&id| id == biome_id).unwrap();

                        let start_long = (biome_index * biome_bits_per_entry as i32) / 64;
                        let start_offset = (biome_index * biome_bits_per_entry as i32) % 64;

                        if start_long < data.len() as i32 {
                            data[start_long as usize] |= (palette_index as i64) << start_offset;
                        }

                        biome_index += 1;
                    }
                }
            }

            PaletteContainer::Indirect {
                bits_per_entry: biome_bits_per_entry,
                palette: palette.into(),
                data,
            }
        };

        Self {
            block_count,
            block_states,
            biomes,
        }
    }
}

// Example implementation of a simple structure
pub struct SimpleHouse {
    stone_id: i32,
    air_id: i32,
    plains_biome_id: i32,
}

impl SimpleHouse {
    pub fn new(stone_id: i32, air_id: i32, biome_id: i32) -> Self {
        Self {
            stone_id,
            air_id,
            plains_biome_id: biome_id,
        }
    }
}

impl Structure for SimpleHouse {
    fn get_block_at(&self, x: i32, y: i32, z: i32) -> i32 {
        // Simple house: stone floor at y=0, stone walls around perimeter
        if y == 0 || y <= 3 && (x == 0 || x == 15 || z == 0 || z == 15) {
            self.stone_id // Stone walls
        } else {
            self.air_id // Air inside and above
        }
    }

    fn get_biome_at(&self, _x: i32, _y: i32, _z: i32) -> i32 {
        self.plains_biome_id // All plains biome
    }
}

#[derive(Debug, Error)]
pub enum ChunkDataError {
    #[error(transparent)]
    Nbt(#[from] NbtEncodeError),
    #[error(transparent)]
    Vec(#[from] LengthPaddedVecEncodeError),
    #[error(transparent)]
    VecEncodeError(#[from] VecEncodeError),
    #[error(transparent)]
    Infallible(#[from] std::convert::Infallible),
}

impl EncodePacketField for ChunkData {
    type Error = ChunkDataError;

    fn encode(&self, bytes: &mut Vec<u8>, protocol_version: u32) -> Result<(), Self::Error> {
        if protocol_version >= ProtocolVersion::V1_21_5.version_number() {
            self.v1_21_5_height_maps.encode(bytes, protocol_version)?;
        } else {
            self.height_maps.encode(bytes, protocol_version)?;
        }

        let mut encoded_data = Vec::<u8>::new();
        self.data.encode(&mut encoded_data, protocol_version)?;

        let mut chunk_sections_payload = Vec::<u8>::new();
        self.data
            .encode(&mut chunk_sections_payload, protocol_version)?;

        let payload_size = VarInt::new(chunk_sections_payload.len() as i32);
        payload_size.encode(bytes, protocol_version)?;

        bytes.extend_from_slice(&chunk_sections_payload);

        self.block_entities.encode(bytes, protocol_version)?;

        Ok(())
    }
}
