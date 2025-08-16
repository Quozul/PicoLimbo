use minecraft_protocol::prelude::*;

#[derive(PacketOut)]
pub struct RegistryEntry {
    entry_id: Identifier,
    /// Whether the entry has any data following.
    has_data: bool,
    /// Entry data. Only present if Has Data is true.
    nbt_bytes: Omitted<Vec<u8>>,
}

impl RegistryEntry {
    pub fn new(entry_id: Identifier, nbt_bytes: Vec<u8>) -> Self {
        Self {
            entry_id,
            has_data: true,
            nbt_bytes: Omitted::Some(nbt_bytes),
        }
    }
}
