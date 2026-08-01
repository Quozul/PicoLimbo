use crate::play::WorldContext;
use crate::play::data::palette_container::PaletteContainer;
use minecraft_protocol::prelude::*;

#[derive(Clone, PacketOut)]
pub struct ChunkSection {
    /// Number of non-air blocks present in the chunk section.
    #[protocol_version(min = V1_14)]
    pub block_count: i16,
    #[protocol_version(min = V26_1)]
    pub fluid_count: i16,
    /// Consists of 4096 entries, representing all the blocks in the chunk section.
    pub block_states: PaletteContainer,
    /// Consists of 64 entries, representing 4×4×4 biome regions in the chunk section.
    #[protocol_version(min = V1_18)]
    pub biomes: PaletteContainer,
    /// Half byte per block
    #[protocol_version(max = V1_13_2)]
    pub block_light: Vec<u8>,
    /// Only if in the Overworld; half byte per block
    #[protocol_version(max = V1_13_2)]
    pub sky_light: Omitted<Vec<u8>>,
}

impl ChunkSection {
    pub const SECTION_SIZE: i32 = 16;

    pub fn void(biome_id: i32) -> Self {
        Self {
            block_count: 0,
            fluid_count: 0,
            block_states: PaletteContainer::blocks_void(),
            biomes: PaletteContainer::single_valued(biome_id),
            block_light: vec![0; 2048],
            sky_light: Omitted::Some(vec![0xFF; 2048]),
        }
    }

    pub fn from_schematic(
        context: &WorldContext,
        section_position: Coordinates,
        biome_id: i32,
        version: ProtocolVersion,
    ) -> ChunkSection {
        if let Some(palette) = context.world.get_section(&section_position) {
            let block_states = PaletteContainer::from_palette(
                palette,
                context.report_id_mapping.as_ref(),
                version,
            );
            let biomes = PaletteContainer::single_valued(biome_id);

            ChunkSection {
                block_count: 4096, // FIXME: Compute this from actual air blocks amount, not a big issue
                fluid_count: 0,
                block_states,
                biomes,
                block_light: vec![0; 2048],
                sky_light: Omitted::Some(vec![0xFF; 2048]),
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
