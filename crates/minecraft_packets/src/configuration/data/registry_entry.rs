use minecraft_protocol::prelude::*;

#[derive(PacketOut)]
pub struct RegistryEntry {
    entry_id: Identifier,
    nbt_bytes: Optional<Vec<u8>>,
}

impl RegistryEntry {
    pub fn new(entry_id: Identifier, nbt_bytes: Vec<u8>) -> Self {
        Self {
            entry_id,
            nbt_bytes: Optional::Some(nbt_bytes),
        }
    }

    pub fn empty(entry_id: Identifier) -> Self {
        Self {
            entry_id,
            nbt_bytes: Optional::None,
        }
    }
}
