use minecraft_protocol::prelude::*;
use std::borrow::Cow;

#[derive(PacketOut)]
pub struct RegistryEntry {
    entry_id: Identifier,
    /// Entry data
    nbt_bytes: Optional<Cow<'static, [u8]>>,
}

impl RegistryEntry {
    pub fn new(entry_id: Identifier, nbt_bytes: Cow<'static, [u8]>) -> Self {
        Self {
            entry_id,
            nbt_bytes: Optional::Some(nbt_bytes),
        }
    }
}
