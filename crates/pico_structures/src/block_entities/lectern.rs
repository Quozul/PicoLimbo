use crate::block_entities::generic::GenericBlockEntity;
use minecraft_protocol::prelude::ProtocolVersion;
use pico_nbt::Value;

/// Lectern block entity with optional book compound (GitHub issue #103).
/// Book data is preserved through schematic load so lecterns keep written content.
#[derive(Clone)]
pub struct LecternBlockEntity {
    nbt: Value,
}

impl LecternBlockEntity {
    pub fn from_nbt(entity_nbt: &Value) -> Self {
        Self {
            nbt: GenericBlockEntity::from_nbt(entity_nbt).to_nbt().clone(),
        }
    }

    pub fn to_version_value(&self, _protocol_version: ProtocolVersion) -> Value {
        self.nbt.clone()
    }

    pub fn has_book(&self) -> bool {
        self.nbt
            .get_compound()
            .map(|c| c.contains_key("Book") || c.contains_key("book"))
            .unwrap_or(false)
    }
}
