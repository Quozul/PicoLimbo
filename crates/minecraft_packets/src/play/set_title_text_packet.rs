use minecraft_protocol::prelude::*;
use pico_text_component::prelude::Component;

#[derive(PacketOut)]
pub struct SetTitleTextPacket {
    text: Component,
}

impl SetTitleTextPacket {
    pub fn new(text: &Component) -> Self {
        Self { text: text.clone() }
    }
}
