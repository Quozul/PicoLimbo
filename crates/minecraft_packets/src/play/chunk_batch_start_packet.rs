use minecraft_protocol::prelude::*;

#[derive(Debug, PacketOut, Default)]
#[packet_id("play/clientbound/minecraft:chunk_batch_start")]
pub struct ChunkBatchStartPacket {}
