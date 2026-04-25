use crate::play::data::block_pos::BlockPos;
use minecraft_protocol::prelude::*;

#[derive(PacketIn)]
pub struct PickItemFromBlockPacket {
    location: BlockPos,
    include_data: bool,
}

impl PickItemFromBlockPacket {
    pub fn location(&self) -> &BlockPos {
        &self.location
    }

    pub fn include_data(&self) -> bool {
        self.include_data
    }
}
