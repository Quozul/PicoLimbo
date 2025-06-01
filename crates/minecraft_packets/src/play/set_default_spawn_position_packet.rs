use minecraft_protocol::prelude::*;

/// This packet is only required starting from 1.19.
#[derive(Debug, PacketOut)]
#[packet_id("play/clientbound/minecraft:set_default_spawn_position")]
pub struct SetDefaultSpawnPosition {
    location: Position,
    #[pvn(755..)]
    angle: f32,
}

impl Default for SetDefaultSpawnPosition {
    fn default() -> Self {
        Self {
            location: Position::new(0.0, 384.0, 0.0),
            angle: 0.0,
        }
    }
}
