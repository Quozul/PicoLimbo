use minecraft_protocol::prelude::PacketOut;

use crate::play::data::block_pos::BlockPos;

#[derive(PacketOut)]
pub struct PickItemFromBlockPacket {
    location: BlockPos,
    include_data: bool,
}
