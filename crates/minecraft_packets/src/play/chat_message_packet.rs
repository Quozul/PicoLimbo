use minecraft_protocol::prelude::*;

#[derive(PacketIn)]
pub struct ChatMessagePacket {
    /// Content of the message. Max length of 256 characters.
    message: String,
    // The rest of the packet (signature since 1.16) is ignored as PicoLimbo does not need it
}

impl ChatMessagePacket {
    pub fn get_command(&self) -> Option<&str> {
        self.message.strip_prefix("/")
    }

    pub fn get_message(&self) -> &str {
        &self.message
    }
}
