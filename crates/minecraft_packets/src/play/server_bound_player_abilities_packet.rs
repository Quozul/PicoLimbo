use minecraft_protocol::prelude::*;

#[derive(PacketIn)]
pub struct ServerBoundPlayerAbilitiesPacket {
    /// Bit mask. 0x02: is flying.
    flags: i8,
}

impl ServerBoundPlayerAbilitiesPacket {
    pub fn is_flying(&self) -> bool {
        (self.flags & 0x02) != 0
    }
}
