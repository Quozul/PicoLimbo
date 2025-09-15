use minecraft_protocol::prelude::*;

#[derive(PacketOut)]
pub struct SetCompressionPacket {
    threshold: VarInt,
}

impl SetCompressionPacket {
    pub fn new(threshold: impl Into<VarInt>) -> Self {
        Self {
            threshold: threshold.into(),
        }
    }
}
