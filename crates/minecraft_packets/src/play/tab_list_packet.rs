use minecraft_protocol::prelude::*;
use pico_text_component::prelude::Component;

#[derive(PacketOut)]
pub struct TabListPacket {
    header: Component,
    footer: Component,
}

impl TabListPacket {
    pub fn new(header: &Component, footer: &Component) -> Self {
        Self {
            header: header.clone(),
            footer: footer.clone(),
        }
    }
}
