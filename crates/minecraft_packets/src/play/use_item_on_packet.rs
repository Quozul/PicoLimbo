use minecraft_protocol::prelude::*;

/// Serverbound: right-click / use item on a block.
#[derive(PacketIn, Debug)]
pub struct UseItemOnPacket {
    pub hand: VarInt,
    pub location: Position,
    pub face: VarInt,
    pub cursor_x: f32,
    pub cursor_y: f32,
    pub cursor_z: f32,
    pub inside_block: bool,
    #[protocol_version(min = V1_19)]
    pub sequence: VarInt,
}

impl UseItemOnPacket {
    pub fn block_pos(&self) -> (i32, i32, i32) {
        self.location.block_coords()
    }
}
