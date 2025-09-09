use minecraft_protocol::prelude::*;
use pico_text_component::prelude::Component;

#[derive(PacketOut)]
pub struct TabListPacket {
    #[pvn(..765)]
    header: String,
    #[pvn(..765)]
    footer: String,
    #[pvn(765..)]
    v1_20_3_header: Nbt,
    #[pvn(765..)]
    v1_20_3_footer: Nbt,
}

impl TabListPacket {
    pub fn new(content: &Component, overlay: &Component) -> Self {
        Self {
            header: content.to_json(),
            footer: overlay.to_json(),
            v1_20_3_header: content.to_nbt(),
            v1_20_3_footer: overlay.to_nbt(),
        }
    }
}
