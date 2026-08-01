use minecraft_protocol::prelude::*;
use pico_text_component::prelude::Component;

/// This packet can be used in the play and configuration state, the structure remains the same
#[derive(PacketOut)]
pub struct DisconnectPacket {
    #[protocol_version(max = V1_20_2)]
    reason: String, // JSON encoded
    #[protocol_version(min = V1_20_3)]
    v1_20_3_reason: pico_nbt::Value, // Nbt starting from 1.20.3 included
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
