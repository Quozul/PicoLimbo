use crate::registry_keys::RegistryKeys;
use minecraft_protocol::prelude::Identifier;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Deserialize)]
pub struct DimensionType {
    height: i32,
    min_y: i32,
}

impl DimensionType {
    pub fn get_height(&self) -> i32 {
        self.height
    }

    pub fn get_min_height(&self) -> i32 {
        self.min_y
    }
}

pub enum RegistryEntryValue {
    Biome,
    CatVariant,
    ChickenVariant,
    CowVariant,
    DamageType,
    DimensionType(DimensionType),
    FrogVariant,
    PaintingVariant,
    PigVariant,
    WolfSoundVariant,
    WolfVariant,
    ZombieNautilusVariant,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct RegistryKey {
    registry: Identifier,
    value: Identifier,
}

impl RegistryKey {
    pub fn of_registry(registry: Identifier) -> Self {
        Self::new(RegistryKeys::Root.id(), registry)
    }

    pub fn new(registry: Identifier, value: Identifier) -> Self {
        Self { registry, value }
    }

    pub fn get_registry(&self) -> &Identifier {
        &self.registry
    }

    pub fn get_value(&self) -> &Identifier {
        &self.value
    }
}

pub struct RegistryEntry {
    value: RegistryEntryValue,
    registry_key: RegistryKey,
    protocol_id: u32,
}

#[derive(Debug, Error)]
pub enum RegistryEntryError {
    #[error("this registry entry is not of the expected type")]
    NotOfType,
}

impl RegistryEntry {
    pub fn get_dimension(&self) -> Result<&DimensionType, RegistryEntryError> {
        match self.value {
            RegistryEntryValue::DimensionType(ref dimension) => Ok(dimension),
            _ => Err(RegistryEntryError::NotOfType),
        }
    }

    pub fn get_protocol_id(&self) -> u32 {
        self.protocol_id
    }

    pub fn get_registry_key(&self) -> &RegistryKey {
        &self.registry_key
    }
}

pub struct Registry {
    entries: HashMap<Identifier, RegistryEntry>,
    registry_key: RegistryKey,
}

impl Registry {
    pub fn get_optional(&self, registry_ref: Identifier) -> Option<&RegistryEntry> {
        self.entries.get(&registry_ref)
    }
}

pub struct RegistryManager {
    registries: HashMap<RegistryKeys, Registry>,
}

impl RegistryManager {
    pub fn from_generated_directory(directory_path: &Path) -> Self {
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
            RegistryKeys::WolfSoundVariant,
            RegistryKeys::WolfVariant,
        ];

        let registries = REGISTRY_KEYS
            .iter()
            .map(|registry_keys| {
                let id = registry_keys.id();
                let sub_path = format!("{}/{}", id.namespace, id.thing);
                let path = directory_path.join(sub_path);
                let read_dir = std::fs::read_dir(path).expect("failed to read directory");
                let mut protocol_id = 0;
                // TODO: Ensure the entries are sorted alphabetically
                let entries = read_dir
                    .map(|dir_entry| {
                        let dir_entry = dir_entry.expect("failed to read directory entry");
                        let path = dir_entry.path();
                        let file_name = path
                            .file_stem()
                            .expect("failed to get file stem")
                            .to_str()
                            .expect("failed to get file stem as str");
                        let registry_key_value = Identifier::new(&id.namespace, file_name);
                        let registry_key = RegistryKey::new(id.clone(), registry_key_value.clone());
                        let value = match registry_keys {
                            RegistryKeys::Biome => RegistryEntryValue::Biome,
                            RegistryKeys::CatVariant => RegistryEntryValue::CatVariant,
                            RegistryKeys::ChickenVariant => RegistryEntryValue::ChickenVariant,
                            RegistryKeys::CowVariant => RegistryEntryValue::CowVariant,
                            RegistryKeys::DamageType => RegistryEntryValue::DamageType,
                            RegistryKeys::DimensionType => {
                                let json_str = std::fs::read_to_string(&path)
                                    .expect("failed to read registries.json");
                                let dimension_type = serde_json::from_str(&json_str)
                                    .expect("failed to deserialize dimension type");
                                RegistryEntryValue::DimensionType(dimension_type)
                            }
                            RegistryKeys::FrogVariant => RegistryEntryValue::FrogVariant,
                            RegistryKeys::PaintingVariant => RegistryEntryValue::PaintingVariant,
                            RegistryKeys::PigVariant => RegistryEntryValue::PigVariant,
                            RegistryKeys::WolfSoundVariant => RegistryEntryValue::WolfSoundVariant,
                            RegistryKeys::WolfVariant => RegistryEntryValue::WolfVariant,
                            _ => panic!("registry key not supported"),
                        };
                        let entry = RegistryEntry {
                            value,
                            registry_key,
                            protocol_id,
                        };
                        protocol_id += 1;
                        (registry_key_value, entry)
                    })
                    .collect::<HashMap<_, _>>();
                let registry_key = RegistryKey::of_registry(registry_keys.id());
                let registry = Registry {
                    entries,
                    registry_key,
                };
                (*registry_keys, registry)
            })
            .collect();
        Self { registries }
    }

    pub fn get_optional(&self, registry_ref: RegistryKeys) -> Option<&Registry> {
        self.registries.get(&registry_ref)
    }
}
