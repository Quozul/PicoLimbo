use minecraft_protocol::prelude::*;

#[derive(PacketIn)]
pub struct ChatCommandPacket {
    command: String,
    // The rest of the packet (signature between 1.16 and 1.19) is ignored as PicoLimbo does not need it
}

impl ChatCommandPacket {
    pub fn get_command(&self) -> &str {
        &self.command
    }
}
