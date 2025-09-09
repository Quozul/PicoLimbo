use minecraft_protocol::prelude::*;

#[derive(PacketOut)]
pub struct TabListPacket {
    pub header: Nbt,
    pub footer: Nbt,
}

impl TabListPacket {
    pub fn new(content: Nbt, overlay: Nbt) -> Self {
        Self {
            header: content,
            footer: overlay,
        }
    }
}
