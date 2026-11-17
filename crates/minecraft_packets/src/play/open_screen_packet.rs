use minecraft_protocol::prelude::*;
use pico_text_component::prelude::Component;

/// Clientbound open container/screen.
#[derive(PacketOut)]
pub struct OpenScreenPacket {
    window_id: VarInt,
    window_type: VarInt,
    window_title: Component,
}

impl OpenScreenPacket {
    pub fn lectern(window_id: i32, menu_type: i32) -> Self {
        Self {
            window_id: VarInt::new(window_id),
            window_type: VarInt::new(menu_type),
            window_title: Component::new("Lectern"),
        }
    }
}
