use crate::registry_keys::RegistryKeys;
use pico_identifier::Identifier;
use serde::Serialize;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize)]
pub struct RegistryKey {
    registry: Identifier,
    value: Identifier,
}

impl RegistryKey {
    /// Key for a registry such as "minecraft:worldgen/biome" registry is in the "minecraft:root" registry
    pub fn of_registry(registry: Identifier) -> Self {
        Self::new(RegistryKeys::Root.id(), registry)
    }

    /// Key for a value in a registry such as "minecraft:plains" is in the "minecraft:worldgen/biome" registry
    pub const fn new(registry: Identifier, value: Identifier) -> Self {
        Self { registry, value }
    }

    pub const fn get_registry(&self) -> &Identifier {
        &self.registry
    }

    pub const fn get_value(&self) -> &Identifier {
        &self.value
    }
}
