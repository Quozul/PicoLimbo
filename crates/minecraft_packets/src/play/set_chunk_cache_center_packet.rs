use minecraft_protocol::prelude::*;

#[derive(PacketOut)]
pub struct SetCenterChunkPacket {
    chunk_x: VarInt,
    chunk_z: VarInt,
}

impl SetCenterChunkPacket {
    pub fn new(chunk_x: i32, chunk_z: i32) -> Self {
        Self {
            chunk_x: VarInt::new(chunk_x),
            chunk_z: VarInt::new(chunk_z),
        }
    }
}
