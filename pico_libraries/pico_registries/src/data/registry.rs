use crate::data::registry_entry::RegistryEntry;
use crate::data::registry_entry_value::RegistryEntryValue;
use crate::data::registry_key::RegistryKey;
use crate::data::tag::Tag;
use crate::registry_keys::RegistryKeys;
use pico_identifier::Identifier;
use serde::Serialize;
use std::collections::HashMap;
use std::fs::DirEntry;
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
    /// Gets a registry entry
    ///
    /// # Errors
    /// Return an error if the entry is not found
    pub fn get(&self, registry_ref: &Identifier) -> crate::Result<&RegistryEntry> {
        self.entries
            .get(registry_ref)
            .ok_or(crate::Error::UnknownRegistryEntry)
    }

    #[must_use]
    pub fn try_get(&self, registry_ref: &Identifier) -> Option<&RegistryEntry> {
        self.entries.get(registry_ref)
    }

    /// Load the registry from a directory
    ///
    /// # Errors
    /// Returns an error if it fails to load a registry
    pub fn load(registry_keys: &RegistryKeys, resource_path: &Path) -> crate::Result<Self> {
        let entries = Self::load_entries(registry_keys, resource_path)?;
        let tags = Self::load_tags(registry_keys, resource_path)?;
        let key = RegistryKey::of_registry(registry_keys.id());
        Ok(Self { entries, key, tags })
    }

    #[must_use]
    pub fn get_entries(&self) -> Vec<&RegistryEntry> {
        let mut entries = self.entries.values().collect::<Vec<_>>();
        entries.sort_by_key(|entry| entry.get_protocol_id());
        entries
    }

    #[must_use]
    pub const fn get_registry_key(&self) -> &RegistryKey {
        &self.key
    }

    #[must_use]
    pub fn get_tag_identifiers(&self) -> Vec<&Identifier> {
        self.tags.keys().collect()
    }

    /// Get a tag
    ///
    /// # Errors
    /// Returns an error if the tag is not found
    pub fn get_tag(&self, identifier: &Identifier) -> crate::Result<&Tag> {
        self.tags
            .get(identifier)
            .ok_or(crate::Error::UnknownTagEntry)
    }

    fn load_entries(
        registry_keys: &RegistryKeys,
        resource_path: &Path,
    ) -> crate::Result<HashMap<Identifier, RegistryEntry>> {
        let id = registry_keys.id();
        let sub_path = format!("{}/{}", id.namespace, id.thing);
        let path = resource_path.join(sub_path);
        let read_dir = std::fs::read_dir(path)?;

        let mut entries: Vec<_> = read_dir.collect::<Result<Vec<_>, _>>()?;

        entries.sort_by_key(DirEntry::file_name);

        let mut protocol_id = 0;
        entries
            .into_iter()
            .map(|dir_entry| -> crate::Result<(Identifier, RegistryEntry)> {
                let path = dir_entry.path();
                let json_str = std::fs::read_to_string(&path)?;
                let file_name = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .ok_or_else(file_stem_error)?;
                let registry_key_value = Identifier::new(&id.namespace, file_name)?;
                let registry_key = RegistryKey::new(id.clone(), registry_key_value.clone());
                let value = match registry_keys {
                    RegistryKeys::DimensionType => {
                        let dimension_type = serde_json::from_str(&json_str)?;
                        RegistryEntryValue::DimensionType(dimension_type)
                    }
                    _ => RegistryEntryValue::Other,
                };
                let json_data = serde_json::from_str(&json_str)?;

                let nbt_value = pico_nbt2::json_to_nbt(json_data)?;

                let entry = RegistryEntry::new(value, nbt_value, registry_key, protocol_id);
                protocol_id += 1;
                Ok((registry_key_value, entry))
            })
            .collect()
    }

    fn load_tags(
        registry_keys: &RegistryKeys,
        resource_path: &Path,
    ) -> crate::Result<HashMap<Identifier, Tag>> {
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
            .map(|dir_entry| -> crate::Result<(Identifier, Tag)> {
                let path = dir_entry.path();
                let json_str = std::fs::read_to_string(path)?;
                let tag = serde_json::from_str::<Tag>(&json_str)?;
                // TODO: Find a cleaner way to make this conversion from path to identifier
                // TODO: Handle \ on Windows which should become / in the tag identifier
                let file_no_ext = path.strip_prefix(&tag_group_path)?.with_extension("");
                let file_stem = file_no_ext.to_str().ok_or_else(file_stem_error)?;
                let tag_identifier = Identifier::new(&registry_keys.id().namespace, file_stem)?;
                Ok((tag_identifier, tag))
            })
            .collect()
    }
}

fn file_stem_error() -> std::io::Error {
    std::io::Error::new(
        std::io::ErrorKind::InvalidData,
        "failed to convert file stem to string",
    )
}
