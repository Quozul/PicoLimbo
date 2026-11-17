use minecraft_protocol::prelude::*;

/// Clientbound block update (single block).
/// Lightweight alternative to resending a full chunk section for sparse edits.
#[derive(PacketOut)]
pub struct BlockUpdatePacket {
    location: Position,
    block_id: VarInt,
}

impl BlockUpdatePacket {
    pub fn new(x: i32, y: i32, z: i32, block_state_id: i32) -> Self {
        Self {
            location: Position::new(f64::from(x), f64::from(y), f64::from(z)),
            block_id: VarInt::new(block_state_id),
        }
    }
}
