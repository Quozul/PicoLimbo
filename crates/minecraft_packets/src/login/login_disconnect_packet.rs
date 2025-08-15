use minecraft_protocol::prelude::*;
use pico_text_component::prelude::PlainText;

#[derive(Debug, PacketOut)]
pub struct LoginDisconnectPacket {
    pub reason: String,
}

impl Default for LoginDisconnectPacket {
    fn default() -> Self {
        Self {
            reason: r#"{"text":"Disconnected"}"#.to_owned(),
        }
    }
}

impl LoginDisconnectPacket {
    pub fn text(text: impl Into<String>) -> LoginDisconnectPacket {
        let component = PlainText::new(text);
        Self {
            reason: component.to_json(),
        }
    }
}
