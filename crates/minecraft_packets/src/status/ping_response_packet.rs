use minecraft_protocol::prelude::*;

#[derive(PacketOut)]
pub struct PongResponsePacket {
    pub timestamp: i64,
}
