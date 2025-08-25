use minecraft_protocol::prelude::*;

/// This packet is only required starting from 1.19.
#[derive(PacketOut)]
pub struct SetDefaultSpawnPositionPacket {
    location: Position,
    #[pvn(755..)]
    angle: f32,
}

impl SetDefaultSpawnPositionPacket {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self {
            location: Position::new(x, y, z),
            angle: 0.0,
        }
    }
}
