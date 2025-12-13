use crate::data::registry_entry::RegistryEntry;
use crate::data::registry_entry_value::RegistryEntryValue;
use crate::data::registry_key::RegistryKey;
use crate::data::tag::Tag;
use crate::registry_keys::RegistryKeys;
use pico_identifier::Identifier;
use serde::Serialize;
use std::collections::HashMap;
use std::path::Path;
use walkdir::WalkDir;

#[derive(Debug, Serialize)]
pub struct Registry {
    entries: HashMap<Identifier, RegistryEntry>,
    key: RegistryKey,
    /// Name of the tag mapped to the tag
    tags: HashMap<Identifier, Tag>,
}

impl Registry {
    pub fn get_optional(&self, registry_ref: &Identifier) -> Option<&RegistryEntry> {
        self.entries.get(registry_ref)
    }

    pub fn load(registry_keys: RegistryKeys, resource_path: &Path) -> Self {
        let entries = Self::load_entries(registry_keys, resource_path);
        let tags = Self::load_tags(registry_keys, resource_path);
        let key = RegistryKey::of_registry(registry_keys.id());
        Self { entries, key, tags }
    }

    fn load_entries(
        registry_keys: RegistryKeys,
        resource_path: &Path,
    ) -> HashMap<Identifier, RegistryEntry> {
        let id = registry_keys.id();
        let sub_path = format!("{}/{}", id.namespace, id.thing);
        let path = resource_path.join(sub_path);
        let read_dir = std::fs::read_dir(path).expect("failed to read directory");
        let mut protocol_id = 0;
        // TODO: Ensure the entries are sorted alphabetically
        read_dir
            .map(|dir_entry| {
                let dir_entry = dir_entry.expect("failed to read directory entry");
                let path = dir_entry.path();
                let json_str =
                    std::fs::read_to_string(&path).expect("failed to read registries.json");
                let file_name = path
                    .file_stem()
                    .expect("failed to get file stem")
                    .to_str()
                    .expect("failed to get file stem as str");
                let registry_key_value =
                    Identifier::new(&id.namespace, file_name).expect("invalid identifier");
                let registry_key = RegistryKey::new(id.clone(), registry_key_value.clone());
                let value = match registry_keys {
                    RegistryKeys::Biome => RegistryEntryValue::Biome,
                    RegistryKeys::CatVariant => RegistryEntryValue::CatVariant,
                    RegistryKeys::ChickenVariant => RegistryEntryValue::ChickenVariant,
                    RegistryKeys::CowVariant => RegistryEntryValue::CowVariant,
                    RegistryKeys::DamageType => RegistryEntryValue::DamageType,
                    RegistryKeys::DimensionType => {
                        let dimension_type = serde_json::from_str(&json_str)
                            .expect("failed to deserialize dimension type");
                        RegistryEntryValue::DimensionType(dimension_type)
                    }
                    RegistryKeys::FrogVariant => RegistryEntryValue::FrogVariant,
                    RegistryKeys::PaintingVariant => RegistryEntryValue::PaintingVariant,
                    RegistryKeys::PigVariant => RegistryEntryValue::PigVariant,
                    RegistryKeys::WolfSoundVariant => RegistryEntryValue::WolfSoundVariant,
                    RegistryKeys::WolfVariant => RegistryEntryValue::WolfVariant,
                    RegistryKeys::Timeline => RegistryEntryValue::Timeline,
                    RegistryKeys::ZombieNautilusVariant => {
                        RegistryEntryValue::ZombieNautilusVariant
                    }
                    _ => panic!("registry key not supported"),
                };
                let json_data =
                    serde_json::from_str(&json_str).expect("failed to deserialize registry value");

                let nbt_value =
                    pico_nbt2::json_to_nbt(json_data).expect("Failed to convert nbt to json");

                let entry = RegistryEntry::new(value, nbt_value, registry_key, protocol_id);
                protocol_id += 1;
                (registry_key_value, entry)
            })
            .collect()
    }

    fn load_tags(registry_keys: RegistryKeys, resource_path: &Path) -> HashMap<Identifier, Tag> {
        let tag_group_path = resource_path
            .join(registry_keys.id().namespace)
            .join(registry_keys.get_tag_path());
        WalkDir::new(&tag_group_path)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| {
                e.file_type().is_file()
                    && e.path().extension().and_then(|e| e.to_str()) == Some("json")
            })
            .map(|dir_entry| {
                let path = dir_entry.path();
                let json_str =
                    std::fs::read_to_string(path).expect("failed to read registries.json");
                let tag = serde_json::from_str::<Tag>(&json_str)
                    .expect("failed to deserialize dimension type");
                // TODO: Find a cleaner way to make this conversion from path to identifier
                // TODO: Handle \ on Windows which should become / in the tag identifier
                let file_no_ext = path
                    .strip_prefix(&tag_group_path)
                    .expect("failed to strip prefix")
                    .with_extension("");
                let file_stem = file_no_ext.to_str().expect("failed to convert file stem");
                let tag_identifier = Identifier::new(&registry_keys.id().namespace, file_stem)
                    .expect("invalid identifier");
                (tag_identifier, tag)
            })
            .collect()
    }
}
