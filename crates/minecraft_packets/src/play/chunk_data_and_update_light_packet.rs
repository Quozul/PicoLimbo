use crate::play::data::chunk_context::{VoidChunkContext, WorldContext};
use crate::play::data::chunk_data::ChunkData;
use crate::play::data::light_data::LightData;
use minecraft_protocol::prelude::*;

/// This packet is only mandatory for versions above 1.20.3,
/// thus the packet is only implemented to work on versions after 1.20.3.
/// The GameEventPacket must be sent before sending this one.
#[derive(PacketOut)]
pub struct ChunkDataAndUpdateLightPacket {
    chunk_x: i32,
    chunk_z: i32,

    #[pvn(..755)]
    full_chunk: bool,

    /// If false, the client will recalculate lighting based on the old/new chunk data
    #[pvn(735..751)]
    ignore_old_data: bool,

    /// BitSet with bits (world height in blocks / 16) set to 1 for every 16×16×16 chunk section whose data is included in Data. The least significant bit represents the chunk section at the bottom of the chunk column (from the lowest y to 15 blocks above).
    /// Up until 1.17.1 included
    #[pvn(755..757)]
    v1_17_primary_bit_mask: LengthPaddedVec<u64>, // availableSections bitset?

    #[pvn(..755)]
    primary_bit_mask: VarInt,

    chunk_data: ChunkData,

    /// If edges should be trusted for light updates.
    /// Up until 1.19.4 included
    #[pvn(757..763)]
    trust_edges: bool,

    // TODO: Implement Update Light packet for versions prior to 1.18
    #[pvn(757..)]
    v1_18_light_data: LightData,
}

impl ChunkDataAndUpdateLightPacket {
    pub fn void(context: VoidChunkContext, protocol_version: ProtocolVersion) -> Self {
        let dimension_height = context.dimension_height;
        Self {
            chunk_x: context.chunk_x,
            chunk_z: context.chunk_z,
            v1_17_primary_bit_mask: LengthPaddedVec::default(),
            primary_bit_mask: VarInt::default(),
            full_chunk: true,
            ignore_old_data: false,
            chunk_data: ChunkData::void(context, protocol_version),
            trust_edges: true,
            v1_18_light_data: LightData::new_void(dimension_height),
        }
    }

    pub fn from_structure(
        chunk_context: VoidChunkContext,
        schematic_context: &WorldContext,
        protocol_version: ProtocolVersion,
    ) -> Self {
        let all_sections_bit_mask = 0b1111_1111_1111_1111i32;
        let chunk_x = chunk_context.chunk_x;
        let chunk_z = chunk_context.chunk_z;

        let light_data = match (
            schematic_context
                .world
                .get_chunk_sky_light(chunk_x, chunk_z),
            schematic_context
                .world
                .get_chunk_block_light(chunk_x, chunk_z),
        ) {
            (Some(sky_light), Some(block_light)) => {
                LightData::from_light_data(sky_light, block_light, chunk_context.dimension_height)
            }
            _ => LightData::new_void(chunk_context.dimension_height),
        };

        Self {
            chunk_x,
            chunk_z,
            v1_17_primary_bit_mask: LengthPaddedVec::new(vec![all_sections_bit_mask as u64]),
            primary_bit_mask: VarInt::new(all_sections_bit_mask),
            full_chunk: true,
            ignore_old_data: false,
            chunk_data: ChunkData::from_schematic(
                chunk_context,
                schematic_context,
                protocol_version,
            ),
            trust_edges: true,
            v1_18_light_data: light_data,
        }
    }
}
