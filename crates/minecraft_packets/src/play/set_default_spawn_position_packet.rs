use crate::play::data::block_pos::BlockPos;
use minecraft_protocol::prelude::*;

#[derive(PacketOut)]
struct GlobalPos {
    dimension: Identifier,
    block_pos: BlockPos,
}

impl GlobalPos {
    fn new(dimension: Identifier, block_pos: BlockPos) -> Self {
        Self {
            dimension,
            block_pos,
        }
    }
}

/// This packet is only required starting from 1.19.
#[derive(PacketOut)]
pub struct SetDefaultSpawnPositionPacket {
    #[pvn(..773)]
    location: Position,
    #[pvn(773..)]
    v1_21_9_respawn_data: GlobalPos,
    #[pvn(755..)]
    angle: f32,
    #[pvn(773..)]
    v1_21_9_pitch: f32,
}

impl SetDefaultSpawnPositionPacket {
    pub fn new(spawn_dimension: Dimension, x: f64, y: f64, z: f64) -> Self {
        Self {
            location: Position::new(x, y, z),
            v1_21_9_respawn_data: GlobalPos::new(
                spawn_dimension.identifier(),
                BlockPos::new(x as i32, y as i32, z as i32),
            ),
            angle: 0.0,
            v1_21_9_pitch: 0.0,
        }
    }
}
