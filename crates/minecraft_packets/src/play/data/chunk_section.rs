use crate::play::WorldContext;
use crate::play::data::palette_container::PaletteContainer;
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
        context: &WorldContext,
        section_position: Coordinates,
        biome_id: i32,
    ) -> ChunkSection {
        if let Some(palette) = context.world.get_section(&section_position) {
            let block_states = PaletteContainer::from_palette(palette, &context.report_id_mapping);
            let biomes = PaletteContainer::single_valued(biome_id);

            ChunkSection {
                block_count: 4096,
                block_states,
                biomes,
            }
        } else {
            Self::void(biome_id)
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
