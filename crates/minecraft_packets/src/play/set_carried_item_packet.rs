use minecraft_protocol::prelude::*;

#[derive(PacketIn)]
pub struct SetCarriedItemPacket {
    slot: i16,
}

impl SetCarriedItemPacket {
    pub fn slot(&self) -> i16 {
        self.slot
    }
}
