use crate::play::data::chunk_context::{VoidChunkContext, WorldContext};
use crate::play::data::chunk_section::ChunkSection;
use crate::play::data::encode_as_bytes::EncodeAsBytes;
use blocks_report::get_block_entity_lookup;
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

        // Process block entities for this chunk
        let block_entities_list =
            Self::collect_chunk_block_entities(&chunk_context, schematic_context, protocol_version);

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
        protocol_version: ProtocolVersion,
    ) -> Vec<BlockEntity> {
        let mut block_entities_list = Vec::new();

        // Get the schematic from the world
        let schematic = &schematic_context.world.get_schematic();
        let lookup = get_block_entity_lookup(protocol_version);

        // Iterate through all block entities in the schematic
        for entity_data in schematic.get_block_entities() {
            // Convert schematic-relative position to world position
            let world_x: i32 = entity_data.position.x() + schematic_context.paste_origin.x();
            let world_y: i32 = entity_data.position.y() + schematic_context.paste_origin.y();
            let world_z: i32 = entity_data.position.z() + schematic_context.paste_origin.z();

            // Check if this block entity belongs to the current chunk
            let entity_chunk_x = world_x >> 4; // Divide by 16
            let entity_chunk_z = world_z >> 4;

            if entity_chunk_x == chunk_context.chunk_x && entity_chunk_z == chunk_context.chunk_z {
                // Determine block entity type from the Id tag
                if let Some(id_tag) = entity_data.nbt.find_tag("Id")
                    && let Some(id_str) = id_tag.get_string()
                    && let Some(protocol_id) = lookup.get_type_id(&id_str)
                {
                    let edited_data = if id_str == "minecraft:sign" {
                        Self::fix_sign_nbt(entity_data.nbt.clone(), protocol_version)
                    } else {
                        entity_data.nbt.clone()
                    };

                    block_entities_list.push(BlockEntity::new(
                        world_x,
                        world_y,
                        world_z,
                        VarInt::new(protocol_id),
                        edited_data,
                    ));
                }
            }
        }

        block_entities_list
    }

    /// Modifies NBT in signs to fix differences between protocol versions
    ///
    /// For schematics created in 1.21.4 and below, signs show up with double quotation marks
    /// surrounding each text line in recent versions. For schematics created in 1.21.5 and
    /// above, signs show up with no text in older versions.
    fn fix_sign_nbt(sign_data: Nbt, protocol_version: ProtocolVersion) -> Nbt {
        match sign_data {
            Nbt::Compound { name, value } => {
                let new_value = value
                    .into_iter()
                    .map(|tag| match tag {
                        // Only modify front_text and back_text
                        Nbt::Compound {
                            name: text_name,
                            value: text_value,
                        } if matches!(
                            text_name.as_deref(),
                            Some("front_text") | Some("back_text")
                        ) =>
                        {
                            let new_text = text_value
                                .into_iter()
                                .map(|inner| match inner {
                                    // Find 'messages' NBT list
                                    Nbt::List {
                                        name: msg_name,
                                        value: messages,
                                        ..
                                    } if msg_name.as_deref() == Some("messages") => {
                                        let edited_messages = messages
                                            .into_iter()
                                            .map(|msg| {
                                                let text = msg.get_string().unwrap_or_default();

                                                let processed_text = if protocol_version
                                                    .is_before_inclusive(ProtocolVersion::V1_21_4)
                                                {
                                                    // For 1.21.4 and below, add quotes if not present
                                                    if text.starts_with('"') && text.ends_with('"')
                                                    {
                                                        text.to_string()
                                                    } else {
                                                        format!("\"{}\"", text)
                                                    }
                                                } else {
                                                    // For 1.21.5 and above, remove quotes if present
                                                    text.strip_prefix('"')
                                                        .and_then(|s| s.strip_suffix('"'))
                                                        .map(String::from)
                                                        .unwrap_or_else(|| text.to_string())
                                                };

                                                // Create a new NBT string with processed value
                                                Nbt::String {
                                                    name: msg.get_name(),
                                                    value: processed_text,
                                                }
                                            })
                                            .collect();

                                        // Rebuild x2
                                        Nbt::List {
                                            name: msg_name.clone(),
                                            value: edited_messages,
                                            tag_type: 8,
                                        }
                                    }
                                    other => other,
                                })
                                .collect();

                            // Rebuild x3
                            Nbt::Compound {
                                name: text_name.clone(),
                                value: new_text,
                            }
                        }
                        other => other,
                    })
                    .collect();

                // Rebuild x4
                Nbt::Compound {
                    name: name.clone(),
                    value: new_value,
                }
            }
            other => other,
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
