use minecraft_protocol::prelude::*;
use std::borrow::Cow;

#[derive(PacketOut)]
pub struct RegistryEntry {
    entry_id: Identifier,
    /// Entry data. If omitted, sourced from the selected known packs.
    nbt_bytes: Optional<Cow<'static, [u8]>>,
}

impl RegistryEntry {
    /// nbt_bytes should be none starting 1.21.5 (included)
    pub fn new(entry_id: Identifier, nbt_bytes: Option<Cow<'static, [u8]>>) -> Self {
        Self {
            entry_id,
            nbt_bytes: Optional::from(nbt_bytes),
        }
    }
}
