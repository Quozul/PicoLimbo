use minecraft_protocol::prelude::*;
use pico_text_component::prelude::Component;

/// This packet can be used in the play and configuration state, the structure remains the same
#[derive(PacketOut)]
pub struct DisconnectPacket {
    #[pvn(..765)]
    reason: String, // JSON encoded
    #[pvn(765..)]
    v1_20_3_reason: pico_nbt2::Value, // Nbt starting from 1.20.3 included
}

impl DisconnectPacket {
    pub fn text(text: impl Into<String>) -> DisconnectPacket {
        let component = Component::new(text);
        Self {
            reason: component.to_json(),
            v1_20_3_reason: component.to_nbt(),
        }
    }
}
