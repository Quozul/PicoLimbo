use crate::play::data::palette_container::PaletteContainer;
use crate::play::{Coordinates, SchematicChunkContext};
use minecraft_protocol::prelude::*;

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

impl ChunkSection {
    pub const SECTION_SIZE: i32 = 16;

    pub fn void(biome_id: i32) -> Self {
        Self {
            block_count: 0,
            block_states: PaletteContainer::blocks_void(),
            biomes: PaletteContainer::single_valued(biome_id),
        }
    }

    pub fn from_schematic(
        context: &SchematicChunkContext,
        section_position: Coordinates,
        biome_id: i32,
    ) -> ChunkSection {
        let section_size = Coordinates::new_uniform(Self::SECTION_SIZE);

        let schematic_min = context.paste_origin;
        let section_origin = section_position * section_size;
        let mut ids = Vec::with_capacity(4096);

        for y in 0..Self::SECTION_SIZE {
            for z in 0..Self::SECTION_SIZE {
                for x in 0..Self::SECTION_SIZE {
                    // World position of this block in the section
                    let world_pos = section_origin + Coordinates::new(x, y, z);
                    let schematic_position = world_pos - schematic_min;

                    let value = context.schematic.get_block_state_id(
                        schematic_position.x(),
                        schematic_position.y(),
                        schematic_position.z(),
                    ) as u32;

                    ids.push(value);
                }
            }
        }

        let bpe: u8 = 15;
        let data = pack_direct(&ids, bpe);

        let block_states = PaletteContainer::Direct {
            bits_per_entry: bpe,
            data,
        };
        let biomes = PaletteContainer::single_valued(biome_id);

        ChunkSection {
            block_count: 4096,
            block_states,
            biomes,
        }
    }
}

/// This function only works for 1.16 and after:
/// In prior versions, entries could cross long boundaries, and there was no padding.
pub fn pack_direct(entries: &[u32], bits_per_entry: u8) -> Vec<u64> {
    assert!(bits_per_entry > 0 && bits_per_entry <= 32);
    let bpe = bits_per_entry as usize;
    let epl = 64 / bpe;
    assert!(epl > 0);

    let mask = (1u64 << bits_per_entry) - 1;

    entries
        .chunks(epl)
        .map(|chunk| {
            chunk.iter().enumerate().fold(0u64, |word, (j, &id)| {
                let shift = (j * bpe) as u32;
                word | (((id as u64) & mask) << shift)
            })
        })
        .collect()
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

    #[test]
    fn should_pack_five_bytes() {
        // Given
        let entries = vec![
            1, 2, 2, 3, 4, 4, 5, 6, 6, 4, 8, 0, 7, 4, 3, 13, 15, 16, 9, 14, 10, 12, 0, 2,
        ];
        let expected_longs = vec![0x0020863148418841u64, 0x01018A7260F68C87u64];
        let bits_per_entry = 5;

        // When
        let result = pack_direct(&entries, bits_per_entry);

        // Then
        assert_eq!(expected_longs, result);
    }
}
