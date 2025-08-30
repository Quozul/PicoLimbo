use crate::play::data::chunk_context::{SchematicChunkContext, VoidChunkContext};
use crate::play::data::chunk_section::ChunkSection;
use crate::play::data::coordinates::Coordinates;
use crate::play::data::encode_as_bytes::EncodeAsBytes;
use minecraft_protocol::prelude::*;

#[derive(PacketOut)]
pub struct ChunkData {
    #[pvn(..770)]
    height_maps: Nbt,
    #[pvn(770..)]
    v1_21_5_height_maps: LengthPaddedVec<HeightMap>,

    /// Biome IDs, ordered by x then z then y, in 4×4×4 blocks.
    /// Up until 1.17.1 included
    #[pvn(..757)]
    biomes: LengthPaddedVec<VarInt>,

    /// Size of Data in bytes!
    /// LengthPaddedVec prefixes with the number of elements!
    data: EncodeAsBytes<Vec<ChunkSection>>,
    block_entities: LengthPaddedVec<BlockEntity>,
}

impl ChunkData {
    pub fn void(context: VoidChunkContext) -> Self {
        let long_array_tag = Nbt::LongArray {
            name: Some("MOTION_BLOCKING".to_string()),
            value: vec![0; 37],
        };
        let root_tag = Nbt::Compound {
            name: None,
            value: vec![long_array_tag],
        };

        let section_count =
            context.dimension.height(context.protocol_version) / ChunkSection::SECTION_SIZE;

        Self {
            height_maps: root_tag,
            v1_21_5_height_maps: LengthPaddedVec::new(vec![HeightMap {
                height_map_type: VarInt::new(4), // Motionblock type
                data: LengthPaddedVec::new(vec![0; 37]),
            }]),
            biomes: LengthPaddedVec::new(vec![VarInt::new(127); 1024]),
            data: EncodeAsBytes::new(vec![
                ChunkSection::void(context.biome_index);
                section_count as usize
            ]),
            block_entities: LengthPaddedVec::default(),
        }
    }

    pub fn from_schematic(
        chunk_context: VoidChunkContext,
        schematic_context: &SchematicChunkContext,
    ) -> Self {
        let long_array_tag = Nbt::LongArray {
            name: Some("MOTION_BLOCKING".to_string()),
            value: vec![0; 37],
        };
        let root_tag = Nbt::Compound {
            name: None,
            value: vec![long_array_tag],
        };

        let mut data = Vec::new();
        let negative_section_count = chunk_context
            .dimension
            .min_y(chunk_context.protocol_version)
            .abs()
            / ChunkSection::SECTION_SIZE;
        let positive_section_count = chunk_context
            .dimension
            .height(chunk_context.protocol_version)
            / ChunkSection::SECTION_SIZE
            - negative_section_count;

        for section_y in -negative_section_count..positive_section_count {
            let coordinates =
                Coordinates::new(chunk_context.chunk_x, section_y, chunk_context.chunk_z);
            let section = ChunkSection::from_schematic(
                schematic_context,
                coordinates,
                chunk_context.biome_index,
            );
            data.push(section);
        }

        Self {
            height_maps: root_tag,
            v1_21_5_height_maps: LengthPaddedVec::new(vec![HeightMap {
                height_map_type: VarInt::new(4), // Motionblock type
                data: LengthPaddedVec::new(vec![0; 37]),
            }]),
            data: EncodeAsBytes::new(data),
            biomes: LengthPaddedVec::default(),
            block_entities: LengthPaddedVec::default(),
        }
    }
}

#[derive(PacketOut)]
struct HeightMap {
    /// 1: WORLD_SURFACE
    /// All blocks other than air, cave air and void air. To determine if a beacon beam is obstructed.
    /// 4: MOTION_BLOCKING
    /// "Solid" blocks, except bamboo saplings and cacti; fluids. To determine where to display rain and snow.
    /// 5: MOTION_BLOCKING_NO_LEAVES
    /// Same as MOTION_BLOCKING, excluding leaf blocks.
    height_map_type: VarInt,
    data: LengthPaddedVec<i64>,
}

#[derive(PacketOut)]
pub struct BlockEntity {
    // TODO: Implement BlockEntity
}
