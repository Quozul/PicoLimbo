use minecraft_protocol::prelude::*;

#[derive(Debug, PacketOut)]
#[packet_id("status/clientbound/minecraft:pong_response")]
pub struct PingResponsePacket {
    pub timestamp: i64,
}
