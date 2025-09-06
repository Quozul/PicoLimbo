use minecraft_protocol::prelude::*;
use pico_text_component::prelude::{MiniMessageError, parse_mini_message};

/// Sends the client a raw system message.
/// Introduced in 1.19
#[derive(PacketOut)]
pub struct SystemChatMessagePacket {
    #[pvn(..765)]
    content: String, // JSON encoded
    #[pvn(765..)]
    v1_20_3_content: Nbt, // Nbt starting from 1.20.3 included
    overlay: bool,
}

impl SystemChatMessagePacket {
    pub fn mini_message(content: &str) -> Result<Self, MiniMessageError> {
        let component = parse_mini_message(content)?;
        Ok(Self {
            content: component.to_json(),
            v1_20_3_content: component.to_nbt(),
            overlay: false,
        })
    }
}
