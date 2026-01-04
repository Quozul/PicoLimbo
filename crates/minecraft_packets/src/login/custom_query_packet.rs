use minecraft_protocol::prelude::*;

/// This packet is currently only used to communicate with the Velocity proxy.
#[derive(PacketOut)]
pub struct CustomQueryPacket {
    pub message_id: VarInt,
    pub channel: Identifier,
    pub data: Vec<u8>,
}

impl CustomQueryPacket {
    pub fn velocity_info_channel(message_id: i32) -> Self {
        Self {
            message_id: VarInt::new(message_id),
            channel: Identifier::new_unchecked("velocity", "player_info"),
            data: Vec::new(),
        }
    }
}
