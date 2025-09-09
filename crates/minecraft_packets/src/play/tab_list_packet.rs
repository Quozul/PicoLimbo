use minecraft_protocol::prelude::*;
use pico_text_component::prelude::Component;

#[derive(PacketOut)]
pub struct TabListPacket {
    pub header: Nbt,
    pub footer: Nbt,
}

impl TabListPacket {
    pub fn new(content: &Component, overlay: &Component) -> Self {
        Self {
            header: content.to_nbt(),
            footer: overlay.to_nbt(),
        }
    }
}
