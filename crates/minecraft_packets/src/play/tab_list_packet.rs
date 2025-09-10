use minecraft_protocol::prelude::*;
use pico_text_component::prelude::Component;

#[derive(PacketOut)]
pub struct TabListPacket {
    header: Component,
    footer: Component,
}

impl TabListPacket {
    pub fn new(content: &Component, overlay: &Component) -> Self {
        Self {
            header: content.clone(),
            footer: overlay.clone(),
        }
    }
}
