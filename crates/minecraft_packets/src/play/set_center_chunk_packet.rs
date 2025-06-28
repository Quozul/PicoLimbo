use minecraft_protocol::prelude::*;

#[derive(Debug, PacketOut)]
#[packet_id("play/clientbound/minecraft:set_chunk_cache_center")]
pub struct SetCenterChunkPacket {
    chunk_x: VarInt,
    chunk_z: VarInt,
}

impl SetCenterChunkPacket {
    pub fn new(x: i32, z: i32) -> Self {
        Self {
            chunk_x: x.into(),
            chunk_z: z.into(),
        }
    }
}
