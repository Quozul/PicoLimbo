use crate::play::data::chunk_context::{VoidChunkContext, WorldContext};
use crate::play::data::chunk_section::ChunkSection;
use crate::play::data::encode_as_bytes::EncodeAsBytes;
use blocks_report::{BlockEntityTypeLookup, get_block_entity_lookup};
use minecraft_protocol::prelude::*;

#[derive(PacketOut)]
pub struct ChunkData {
    #[pvn(..770)]
    height_maps: Nbt,
    #[pvn(770..)]
    v1_21_5_height_maps: LengthPaddedVec<HeightMap>,

    /// Biome IDs, ordered by x then z then y, in 4×4×4 blocks.
    /// Up until 1.17.1 included
    #[pvn(751..757)]
    v1_16_2_biomes: LengthPaddedVec<VarInt>,

    /// This array is always of length 1024
    #[pvn(..751)]
    biomes: Vec<i32>,

    data: EncodeAsBytes<Vec<ChunkSection>>,

    // 1.17 and below
    #[pvn(..757)]
    block_entities: LengthPaddedVec<Nbt>,

    // 1.18+
    #[pvn(757..)]
    v1_18_block_entities: LengthPaddedVec<ChunkBlockEntity>,
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

        let section_count = context.dimension_height / ChunkSection::SECTION_SIZE;

        Self {
            height_maps: root_tag,
            v1_21_5_height_maps: LengthPaddedVec::new(vec![HeightMap {
                height_map_type: VarInt::new(4), // Motionblock type
                data: LengthPaddedVec::new(vec![0; 37]),
            }]),
            v1_16_2_biomes: LengthPaddedVec::new(vec![VarInt::new(context.biome_index); 1024]),
            biomes: vec![context.biome_index; 1024],
            data: EncodeAsBytes::new(vec![
                ChunkSection::void(context.biome_index);
                section_count as usize
            ]),
            block_entities: LengthPaddedVec::default(),
            v1_18_block_entities: LengthPaddedVec::default(),
        }
    }

    pub fn from_schematic(
        chunk_context: VoidChunkContext,
        schematic_context: &WorldContext,
        protocol_version: ProtocolVersion,
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
        let negative_section_count =
            chunk_context.dimension_min_y.abs() / ChunkSection::SECTION_SIZE;
        let positive_section_count =
            chunk_context.dimension_height / ChunkSection::SECTION_SIZE - negative_section_count;

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

        let block_entity_lookup = get_block_entity_lookup(protocol_version);

        // Process block entities for this chunk
        let (block_entities_legacy, block_entities) = Self::collect_chunk_block_entities(
            &chunk_context,
            schematic_context,
            &block_entity_lookup,
            protocol_version,
        );

        Self {
            height_maps: root_tag,
            v1_21_5_height_maps: LengthPaddedVec::new(vec![HeightMap {
                height_map_type: VarInt::new(4), // Motionblock type
                data: LengthPaddedVec::new(vec![0; 37]),
            }]),
            v1_16_2_biomes: LengthPaddedVec::new(vec![
                VarInt::new(chunk_context.biome_index);
                1024
            ]),
            biomes: vec![chunk_context.biome_index; 1024],
            data: EncodeAsBytes::new(data),
            block_entities: LengthPaddedVec::new(block_entities_legacy),
            v1_18_block_entities: LengthPaddedVec::new(block_entities),
        }
    }

    fn collect_chunk_block_entities(
        chunk_context: &VoidChunkContext,
        schematic_context: &WorldContext,
        block_entity_lookup: &BlockEntityTypeLookup,
        protocol_version: ProtocolVersion,
    ) -> (Vec<Nbt>, Vec<ChunkBlockEntity>) {
        let mut block_entities = Vec::new();
        let mut v1_18_block_entities = Vec::new();

        // Get pre-computed block entities for this chunk
        let Some(entities) = schematic_context
            .world
            .get_chunk_block_entities(chunk_context.chunk_x, chunk_context.chunk_z)
        else {
            return (block_entities, v1_18_block_entities);
        };

        // Iterate through all block entities
        for entity_data in entities {
            let Some(protocol_id) =
                block_entity_lookup.get_type_id(&entity_data.get_block_entity_type().to_string())
            else {
                continue;
            };

            let nbt = entity_data.to_nbt(protocol_version);

            let coordinates = entity_data.get_position() + schematic_context.paste_origin;

            if protocol_version.is_after_inclusive(ProtocolVersion::V1_18) {
                v1_18_block_entities.push(ChunkBlockEntity::new(
                    coordinates.x(),
                    coordinates.y(),
                    coordinates.z(),
                    VarInt::new(protocol_id),
                    nbt,
                ));
            } else {
                let mut nbt_fields = vec![
                    Nbt::string("id", entity_data.block_entity_type.clone()),
                    Nbt::int("x", coordinates.x()),
                    Nbt::int("y", coordinates.y()),
                    Nbt::int("z", coordinates.z()),
                ];

                if let Nbt::Compound { value, .. } = nbt {
                    nbt_fields.extend(value);
                }

                block_entities.push(Nbt::nameless_compound(nbt_fields));
            }
        }

        (block_entities, v1_18_block_entities)
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
pub struct ChunkBlockEntity {
    /// Packed XZ coordinates within the chunk section (X: 4 bits, Z: 4 bits)
    /// Calculated as: ((x & 15) << 4) | (z & 15)
    packed_xz: u8,
    /// Y coordinate within the chunk section (0-15 for normal sections)
    y: i16,
    /// Type of block entity (VarInt registry ID)
    block_entity_type: VarInt,
    /// NBT data for the block entity
    data: Nbt,
}

impl ChunkBlockEntity {
    /// Creates a new BlockEntity from world coordinates and NBT data
    pub fn new(
        world_x: i32,
        world_y: i32,
        world_z: i32,
        block_entity_type: VarInt,
        data: Nbt,
    ) -> Self {
        // Pack X and Z coordinates (each only needs 4 bits since chunk is 16x16)
        let chunk_x = (world_x & 15) as u8;
        let chunk_z = (world_z & 15) as u8;
        let packed_xz = (chunk_x << 4) | chunk_z;

        Self {
            packed_xz,
            y: world_y as i16,
            block_entity_type,
            data,
        }
    }
}
