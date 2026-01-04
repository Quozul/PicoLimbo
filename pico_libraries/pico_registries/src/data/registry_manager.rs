use crate::data::registry::Registry;
use crate::registry_keys::RegistryKeys;
use std::collections::HashMap;
use std::path::Path;
use tracing::debug;

pub struct RegistryManager {
    registries: HashMap<RegistryKeys, Registry>,
}

impl RegistryManager {
    #[must_use]
    pub const fn builder() -> RegistryManagerBuilder {
        RegistryManagerBuilder::new()
    }

    /// Get a registry
    ///
    /// # Errors
    /// Returns an error if the registry is not found
    pub fn get(&self, registry_ref: &RegistryKeys) -> crate::Result<&Registry> {
        self.registries
            .get(registry_ref)
            .ok_or(crate::Error::UnknownRegistry)
    }

    #[must_use]
    pub fn try_get(&self, registry_ref: &RegistryKeys) -> Option<&Registry> {
        self.registries.get(registry_ref)
    }
}

pub struct RegistryManagerBuilder {
    registry_keys: Vec<RegistryKeys>,
}

impl RegistryManagerBuilder {
    pub const DEFAULT_REGISTRIES: &[RegistryKeys] = &[
        RegistryKeys::Biome,
        RegistryKeys::CatVariant,
        RegistryKeys::ChickenVariant,
        RegistryKeys::CowVariant,
        RegistryKeys::DamageType,
        RegistryKeys::DimensionType,
        RegistryKeys::FrogVariant,
        RegistryKeys::PaintingVariant,
        RegistryKeys::PigVariant,
        RegistryKeys::Timeline,
        RegistryKeys::WolfSoundVariant,
        RegistryKeys::WolfVariant,
        RegistryKeys::ZombieNautilusVariant,
    ];

    #[must_use]
    pub const fn new() -> Self {
        Self {
            registry_keys: Vec::new(),
        }
    }

    /// Register a single registry key
    #[must_use]
    pub fn register(mut self, key: RegistryKeys) -> Self {
        self.registry_keys.push(key);
        self
    }

    /// Register multiple registry keys at once
    #[must_use]
    pub fn register_all(mut self, keys: &[RegistryKeys]) -> Self {
        self.registry_keys.extend_from_slice(keys);
        self
    }

    /// Build the `RegistryManager` by loading all registered registries from the resource path
    #[must_use]
    pub fn load_from_resource_path(self, resource_path: &Path) -> RegistryManager {
        let data_path = resource_path.join("data");
        let registries = self
            .registry_keys
            .iter()
            .filter_map(|registry_key| {
                Registry::load(registry_key, &data_path).map_or_else(
                    |_| {
                        debug!(
                            registry_key = ?registry_key,
                            "Failed to load registry, skipping"
                        );
                        None
                    },
                    |registry| Some((registry_key.clone(), registry)),
                )
            })
            .collect();
        RegistryManager { registries }
    }

    /// Register the default set of registry keys
    #[must_use]
    pub fn with_defaults(self) -> Self {
        self.register_all(Self::DEFAULT_REGISTRIES)
    }
}

impl Default for RegistryManagerBuilder {
    fn default() -> Self {
        Self::new()
    }
}
