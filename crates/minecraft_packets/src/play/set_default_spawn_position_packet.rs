use minecraft_protocol::prelude::*;

/// This packet is only required starting from 1.19.
#[derive(PacketOut)]
pub struct SetDefaultSpawnPositionPacket {
    location: Position,
    #[pvn(755..)]
    angle: f32,
}

impl Default for SetDefaultSpawnPositionPacket {
    fn default() -> Self {
        Self {
            location: Position::new(0.0, 320.0, 0.0),
            angle: 0.0,
        }
    }
}
