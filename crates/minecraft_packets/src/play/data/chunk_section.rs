use crate::play::data::palette_container::{PaletteContainer, PaletteContainerError};
use minecraft_protocol::prelude::*;
use pico_structures::prelude::Structure;
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct ChunkSection {
    /// Number of non-air blocks present in the chunk section.
    pub block_count: i16,
    /// Consists of 4096 entries, representing all the blocks in the chunk section.
    pub block_states: PaletteContainer,
    /// Consists of 64 entries, representing 4×4×4 biome regions in the chunk section.
    pub biomes: PaletteContainer,
}

impl ChunkSection {
    pub fn void(biome_id: i32) -> Self {
        Self {
            block_count: 0,
            block_states: PaletteContainer::blocks_void(),
            biomes: PaletteContainer::single_valued(biome_id.into()),
        }
    }

    pub fn from_structure(structure: Structure, biome_id: i32) -> Self {
        // Collect all unique blocks and biomes
        let mut block_palette = Vec::new();

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

        // Determine bits per entry based on palette size
        let block_bits_per_entry = if block_palette.len() == 1 {
            0 // Single valued
        } else {
            std::cmp::max(4, (block_palette.len() as f64).log2().ceil() as u8)
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

        let biomes = PaletteContainer::SingleValued {
            bits_per_entry: 0,
            value: VarInt::new(biome_id),
            data: Vec::new(),
        };

        Self {
            block_count,
            block_states,
            biomes,
        }
    }
}

#[derive(Error, Debug)]
pub enum ChunkSectionError {
    #[error("error while decoding a palette")]
    EncodeError,
    #[error("invalid palette container error")]
    Infallible,
    #[error("error while decoding a palette container")]
    PaletteContainerError,
}

impl From<std::convert::Infallible> for ChunkSectionError {
    fn from(_: std::convert::Infallible) -> Self {
        ChunkSectionError::Infallible
    }
}

impl<T: DecodePacketField> From<LengthPaddedVecDecodeError<T>> for ChunkSectionError {
    fn from(_: LengthPaddedVecDecodeError<T>) -> Self {
        ChunkSectionError::EncodeError
    }
}

impl From<PaletteContainerError> for ChunkSectionError {
    fn from(_: PaletteContainerError) -> Self {
        ChunkSectionError::PaletteContainerError
    }
}

impl EncodePacketField for ChunkSection {
    type Error = ChunkSectionError;

    fn encode(&self, bytes: &mut Vec<u8>, protocol_version: u32) -> Result<(), Self::Error> {
        self.block_count.encode(bytes, protocol_version)?;
        self.block_states.encode(bytes, protocol_version)?;
        self.biomes.encode(bytes, protocol_version)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn expected_snapshots() -> HashMap<u32, Vec<u8>> {
        HashMap::from([
            (
                770,
                vec![
                    /* Block count */
                    0x00, 0x00,
                    /* Block states */
                    /* Bits Per Entry */
                    0x00, /* Palette */
                    /* Value */
                    0x00, /* Biomes */
                    /* Bits Per Entry */
                    0x00, /* Value */
                    0x7F,
                ],
            ),
            (
                769,
                vec![
                    /* Block count */
                    0x00, 0x00,
                    /* Block states */
                    /* Bits Per Entry */
                    0x00, /* Palette */
                    /* Value */
                    0x00, /* Data Array Length */
                    0x00, /* Biomes */
                    /* Bits Per Entry */
                    0x00, /* Value */
                    0x7F, /* Data Array Length */
                    0x00,
                ],
            ),
        ])
    }

    fn create_packet() -> ChunkSection {
        let biome_id = 127;
        ChunkSection::void(biome_id)
    }

    #[test]
    fn chunk_data_and_update_light_packets() {
        let snapshots = expected_snapshots();

        for (version, expected_bytes) in snapshots {
            let packet = create_packet();
            let mut bytes = Vec::new();
            packet.encode(&mut bytes, version).unwrap();
            assert_eq!(expected_bytes, bytes, "Mismatch for version {version}");
        }
    }
}
