use minecraft_protocol::prelude::*;

#[derive(Debug, PacketOut)]
pub struct PongResponsePacket {
    pub timestamp: i64,
}
