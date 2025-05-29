use minecraft_protocol::prelude::*;
use pico_text_component::prelude::PlainText;

/// Sends the client a raw system message.
/// Introduced in 1.19
#[derive(Debug, PacketOut)]
#[packet_id("play/clientbound/minecraft:system_chat")]
pub struct SystemChatMessage {
    #[pvn(..765)]
    content: String, // JSON encoded
    #[pvn(765..)]
    v1_20_3_content: Nbt, // Nbt starting from 1.20.3 included
    overlay: bool,
}

impl SystemChatMessage {
    pub fn plain_text<S>(content: S) -> Self
    where
        S: Into<String>,
    {
        let component = PlainText::new(content);
        Self {
            content: component.to_json(),
            v1_20_3_content: component.to_nbt(),
            overlay: false,
        }
    }
}
