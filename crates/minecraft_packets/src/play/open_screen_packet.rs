use minecraft_protocol::prelude::*;
use pico_text_component::prelude::Component;

/// Clientbound open container/screen. Lectern uses menu type ~17 on modern protocols.
#[derive(PacketOut)]
pub struct OpenScreenPacket {
    window_id: VarInt,
    window_type: VarInt,
    window_title: Component,
}

impl OpenScreenPacket {
    /// Open a lectern reading UI (no book placed in player hand).
    pub fn lectern(window_id: i32) -> Self {
        Self {
            window_id: VarInt::new(window_id),
            // minecraft:lectern menu type is protocol_id 17 on 1.20.3+ / 26.x registries.
            window_type: VarInt::new(17),
            window_title: Component::new("Lectern"),
        }
    }
}
