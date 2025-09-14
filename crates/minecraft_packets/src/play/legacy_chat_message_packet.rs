use minecraft_protocol::prelude::*;
use pico_text_component::prelude::Component;

/// This packet has no equivalent since 1.19 included
/// It has been split into 3 packets:
/// - Disguised Chat Message
/// - Player Chat Message
/// - System Chat Message
#[derive(PacketOut)]
pub struct LegacyChatMessagePacket {
    /// JSON encoded text component
    content: String,
    /// 0: chat (chat box), 1: system message (chat box), 2: game info (above hotbar)
    #[pvn(47..)]
    position: u8,
    /// Used by the Notchian client for the disableChat launch option. Setting both longs to 0 will always display the message regardless of the setting.
    #[pvn(735..)]
    sender: UuidAsString,
}

impl LegacyChatMessagePacket {
    pub fn component(component: &Component) -> Self {
        Self {
            content: component.to_json(),
            position: 1,
            sender: UuidAsString::default(),
        }
    }
}
