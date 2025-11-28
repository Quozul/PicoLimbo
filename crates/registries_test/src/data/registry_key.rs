use crate::registry_keys::RegistryKeys;
use pico_identifier::Identifier;
use serde::Serialize;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize)]
pub struct RegistryKey {
    registry: Identifier,
    value: Identifier,
}

impl RegistryKey {
    pub fn of_registry(registry: Identifier) -> Self {
        Self::new(RegistryKeys::Root.id(), registry)
    }

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
