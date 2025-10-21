use crate::play::data::chunk_context::{VoidChunkContext, WorldContext};
use crate::play::data::chunk_section::ChunkSection;
use crate::play::data::encode_as_bytes::EncodeAsBytes;
use blocks_report::{BlockEntityTypeLookup, get_block_entity_lookup};
use minecraft_protocol::prelude::*;
use pico_structures::prelude::IntermediateBlockEntityData;

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

        let section_count = context.dimension.height() / ChunkSection::SECTION_SIZE;

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
            chunk_context.dimension.min_y().abs() / ChunkSection::SECTION_SIZE;
        let positive_section_count =
            chunk_context.dimension.height() / ChunkSection::SECTION_SIZE - negative_section_count;

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
        let block_entities_list = Self::collect_chunk_block_entities(
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
            block_entities: LengthPaddedVec::new(block_entities_list),
        }
    }

    fn collect_chunk_block_entities(
        chunk_context: &VoidChunkContext,
        schematic_context: &WorldContext,
        block_entity_lookup: &BlockEntityTypeLookup,
        protocol_version: ProtocolVersion,
    ) -> Vec<BlockEntity> {
        let mut block_entities_list = Vec::new();

        // Get pre-computed block entities for this chunk
        let Some(entities) = schematic_context
            .world
            .get_chunk_block_entities(chunk_context.chunk_x, chunk_context.chunk_z)
        else {
            return block_entities_list;
        };

        // Iterate through all block entities
        for entity_data in entities {
            // Convert schematic-relative position to world position
            let world_x = entity_data.world_x + schematic_context.paste_origin.x();
            let world_y = entity_data.world_y + schematic_context.paste_origin.y();
            let world_z = entity_data.world_z + schematic_context.paste_origin.z();

            // Look up protocol ID
            let Some(protocol_id) = block_entity_lookup.get_type_id(&entity_data.block_entity_type)
            else {
                continue;
            };

            // Convert intermediate format to protocol-specific NBT
            let nbt = Self::intermediate_to_nbt(&entity_data.nbt, protocol_version);

            block_entities_list.push(BlockEntity::new(
                world_x,
                world_y,
                world_z,
                VarInt::new(protocol_id),
                nbt,
            ));
        }

        block_entities_list
    }

    fn intermediate_to_nbt(
        data: &IntermediateBlockEntityData,
        protocol_version: ProtocolVersion,
    ) -> Nbt {
        match data {
            IntermediateBlockEntityData::Sign {
                front_messages,
                back_messages,
                front_color,
                back_color,
                front_glowing,
                back_glowing,
                is_waxed,
            } => {
                // Format messages based on protocol version
                let format_messages = |messages: &[String; 4]| {
                    messages
                        .iter()
                        .map(|msg| {
                            let text =
                                if protocol_version.is_before_inclusive(ProtocolVersion::V1_21_4) {
                                    // Add quotes for 1.21.4 and below
                                    format!("\"{}\"", msg)
                                } else {
                                    // No quotes for 1.21.5+
                                    msg.clone()
                                };
                            Nbt::String {
                                name: None,
                                value: text,
                            }
                        })
                        .collect()
                };

                let front_text = Nbt::Compound {
                    name: Some("front_text".to_string()),
                    value: vec![
                        Nbt::String {
                            name: Some("color".to_string()),
                            value: front_color.clone(),
                        },
                        Nbt::Byte {
                            name: Some("has_glowing_text".to_string()),
                            value: if *front_glowing { 1 } else { 0 },
                        },
                        Nbt::List {
                            name: Some("messages".to_string()),
                            value: format_messages(front_messages),
                            tag_type: 8,
                        },
                    ],
                };

                let back_text = Nbt::Compound {
                    name: Some("back_text".to_string()),
                    value: vec![
                        Nbt::String {
                            name: Some("color".to_string()),
                            value: back_color.clone(),
                        },
                        Nbt::Byte {
                            name: Some("has_glowing_text".to_string()),
                            value: if *back_glowing { 1 } else { 0 },
                        },
                        Nbt::List {
                            name: Some("messages".to_string()),
                            value: format_messages(back_messages),
                            tag_type: 8,
                        },
                    ],
                };

                Nbt::Compound {
                    name: None,
                    value: vec![
                        front_text,
                        back_text,
                        Nbt::Byte {
                            name: Some("is_waxed".to_string()),
                            value: if *is_waxed { 1 } else { 0 },
                        },
                    ],
                }
            }
            IntermediateBlockEntityData::Generic { nbt } => nbt.clone(),
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

impl BlockEntity {
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
