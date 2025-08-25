use crate::play::data::palette_container::PaletteContainer;
use minecraft_protocol::prelude::*;
use pico_structures::prelude::Schematic;

#[derive(Clone, PacketOut)]
pub struct ChunkSection {
    /// Number of non-air blocks present in the chunk section.
    pub block_count: i16,
    /// Consists of 4096 entries, representing all the blocks in the chunk section.
    pub block_states: PaletteContainer,
    /// Consists of 64 entries, representing 4×4×4 biome regions in the chunk section.
    #[pvn(757..)]
    pub biomes: PaletteContainer,
}

const SECTION_HEIGHT: i64 = 16;
const SECTION_WIDTH: i64 = 16;

impl ChunkSection {
    pub fn void(biome_id: i32) -> Self {
        Self {
            block_count: 0,
            block_states: PaletteContainer::blocks_void(),
            biomes: PaletteContainer::single_valued(biome_id),
        }
    }

    pub fn from_structure(
        structure: &Schematic,
        paste_origin: (i32, i32, i32),
        chunk_x: i32,
        section_y: i32,
        chunk_z: i32,
        biome_id: i32,
    ) -> ChunkSection {
        let block_count: i16 = structure.get_solid_block_count() as i16;
        let structure_palette = structure.get_palette();
        // FIXME: Figure out why this works for 4 and 8 only
        let bits_per_block: i64 = 8; //(structure_palette.len() as f32).log2().ceil() as i64;

        let total_bits = (16 * 16 * 16) * bits_per_block as usize;
        let data_length = total_bits.div_ceil(64);
        let mut data = vec![0i64; data_length];
        let individual_value_mask = (1 << bits_per_block) - 1;

        let (paste_x, paste_y, paste_z) = paste_origin;

        for y in 0..SECTION_HEIGHT {
            for z in 0..SECTION_WIDTH {
                for x in 0..SECTION_WIDTH {
                    let world_x = (chunk_x as i64 * SECTION_WIDTH + x) as i32;
                    let world_y = (section_y as i64 * SECTION_HEIGHT + y) as i32;
                    let world_z = (chunk_z as i64 * SECTION_WIDTH + z) as i32;

                    let schematic_x = world_x - paste_x;
                    let schematic_y = world_y - paste_y;
                    let schematic_z = world_z - paste_z;

                    let mut value: i64 =
                        structure.get_block_at(schematic_x, schematic_y, schematic_z) as i64;
                    value &= individual_value_mask;

                    let block_number: i64 = (((y * SECTION_HEIGHT) + z) * SECTION_WIDTH) + x;
                    let start_long: i64 = (block_number * bits_per_block) / 64;
                    let start_offset: i64 = (block_number * bits_per_block) % 64;
                    let end_long: i64 = ((block_number + 1) * bits_per_block - 1) / 64;

                    data[start_long as usize] |= value << start_offset;
                    if start_long != end_long {
                        data[end_long as usize] |= value >> (64 - start_offset);
                    }
                }
            }
        }

        let block_states = PaletteContainer::Indirect {
            bits_per_entry: bits_per_block as u8,
            palette: LengthPaddedVec::new(structure_palette.clone()),
            data,
        };

        let biomes = PaletteContainer::single_valued(biome_id);

        ChunkSection {
            block_count,
            block_states,
            biomes,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn expected_snapshots() -> HashMap<i32, Vec<u8>> {
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
            let mut bytes = BinaryWriter::default();
            packet
                .encode(&mut bytes, ProtocolVersion::from(version))
                .unwrap();
            let bytes = bytes.into_inner();
            assert_eq!(expected_bytes, bytes, "Mismatch for version {version}");
        }
    }
}
