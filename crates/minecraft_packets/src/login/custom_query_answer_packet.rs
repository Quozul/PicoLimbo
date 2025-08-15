use minecraft_protocol::prelude::*;

/// This packet is currently only used to communicate with the Velocity proxy.
#[derive(Debug, PacketIn)]
pub struct CustomQueryAnswerPacket {
    pub message_id: VarInt,
    pub is_present: bool,
    pub data: Vec<u8>,
}
