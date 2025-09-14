use minecraft_protocol::prelude::*;
use pico_text_component::prelude::Component;

#[derive(PacketOut)]
pub struct SetSubtitleTextPacket {
    subtitle: Component,
}

impl SetSubtitleTextPacket {
    pub fn new(subtitle: &Component) -> Self {
        Self {
            subtitle: subtitle.clone(),
        }
    }
}
