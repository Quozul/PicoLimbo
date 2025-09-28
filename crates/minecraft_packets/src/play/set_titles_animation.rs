use minecraft_protocol::prelude::*;

#[derive(PacketOut)]
pub struct SetTitlesAnimationPacket {
    fade_in: i32,
    stay: i32,
    fade_out: i32,
}

impl SetTitlesAnimationPacket {
    pub fn new(fade_in: i32, stay: i32, fade_out: i32) -> Self {
        Self {
            fade_in,
            stay,
            fade_out,
        }
    }
}
