use crate::play::data::block_pos::BlockPos;
use minecraft_protocol::prelude::*;

#[derive(PacketOut)]
pub struct GlobalPos {
    dimension: Identifier,
    block_pos: BlockPos,
}

impl GlobalPos {
    pub fn new(dimension: Identifier, block_pos: BlockPos) -> Self {
        Self {
            dimension,
            block_pos,
        }
    }
}
