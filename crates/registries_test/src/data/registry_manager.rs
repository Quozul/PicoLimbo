use crate::data::registry::Registry;
use crate::registry_keys::RegistryKeys;
use std::collections::HashMap;
use std::path::Path;

pub struct RegistryManager {
    registries: HashMap<RegistryKeys, Registry>,
}

impl RegistryManager {
    pub fn from_resource_path(resource_path: &Path) -> Self {
        // A list of all known registries to load
        const REGISTRY_KEYS: &[RegistryKeys] = &[
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

        let data_path = resource_path.join("data");
        let registries = REGISTRY_KEYS
            .iter()
            .map(|registry_keys| {
                let registry = Registry::load(*registry_keys, &data_path);
                (*registry_keys, registry)
            })
            .collect();
        Self { registries }
    }

    pub fn get_optional(&self, registry_ref: RegistryKeys) -> Option<&Registry> {
        self.registries.get(&registry_ref)
    }
}
