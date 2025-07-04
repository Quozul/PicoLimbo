use minecraft_protocol::prelude::*;
use pico_text_component::prelude::PlainText;

/// This packet has no equivalent since 1.19 included
/// It has been split into 3 packets:
/// - Disguised Chat Message
/// - Player Chat Message
/// - System Chat Message
#[derive(Debug, PacketOut)]
#[packet_id("play/clientbound/minecraft:legacy_chat_message")]
pub struct LegacyChatMessage {
    /// JSON encoded text component
    content: String,
    /// 0: chat (chat box), 1: system message (chat box), 2: game info (above hotbar)
    #[pvn(47..)]
    position: u8,
    /// Used by the Notchian client for the disableChat launch option. Setting both longs to 0 will always display the message regardless of the setting.
    #[pvn(735..)]
    sender: Uuid,
}

impl LegacyChatMessage {
    pub fn system<S>(content: S) -> Self
    where
        S: Into<String>,
    {
        let component = PlainText::new(content);
        Self {
            content: component.to_json(),
            position: 1,
            sender: Uuid::nil(),
        }
    }
}
