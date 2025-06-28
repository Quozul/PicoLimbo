use minecraft_protocol::prelude::*;

#[derive(Debug, PacketOut, Default)]
#[packet_id("play/clientbound/minecraft:chunk_batch_finished")]
pub struct ChunkBatchFinishedPacket {
    /// Number of chunks.
    batch_size: VarInt,
}

impl ChunkBatchFinishedPacket {
    pub fn new(batch_size: i32) -> Self {
        Self {
            batch_size: batch_size.into(),
        }
    }
}
