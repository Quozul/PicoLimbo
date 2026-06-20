use crate::data::registry::{NbtRegistryData, NbtTagData, Registry};
use crate::registry_keys::RegistryKeys;
use crate::reports::registries_report::RegistriesReport;
use pico_nbt::{IndexMap, NbtOptions, from_path_with_options, from_value};
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

        // Parsed once for the whole manager, not per registry - the report is a
        // few hundred kilobytes. Missing or unreadable reports are not fatal:
        // registries that have an entry directory work without it, only tags on
        // directory-less registries lose their IDs.
        let report = RegistriesReport::from_resource_path(resource_path).map_or_else(
            |error| {
                debug!(?error, "Failed to load registries report, continuing without");
                None
            },
            Some,
        );

        let registries = self
            .registry_keys
            .iter()
            .filter_map(|registry_key| {
                let report_protocol_ids = report
                    .as_ref()
                    .and_then(|report| report.registries.get(&registry_key.id()))
                    .map(|registry| {
                        registry
                            .entries
                            .iter()
                            .map(|(identifier, entry)| (identifier.clone(), entry.protocol_id))
                            .collect()
                    })
                    .unwrap_or_default();

                Registry::load_from_resource_path(registry_key, &data_path, report_protocol_ids)
                    .map_or_else(
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

    /// Build the `RegistryManager` by loading all registered registries from NBT files
    #[must_use]
    pub fn load_from_nbt_files(self, base_path: &Path) -> RegistryManager {
        let nbt_options = NbtOptions::new().nameless_root(true);

        let registries_data = {
            let registries_nbt_path = base_path.join("registries.nbt");
            match from_path_with_options(&registries_nbt_path, nbt_options) {
                Ok((_name, value)) => from_value::<IndexMap<String, NbtRegistryData>>(value)
                    .map_err(|e| debug!("Failed to parse registries NBT: {}", e))
                    .ok(),
                Err(e) => {
                    debug!("Failed to load registries NBT: {}", e);
                    None
                }
            }
        };

        let tags_data = {
            let tags_nbt_path = base_path.join("tags.nbt");
            match from_path_with_options(&tags_nbt_path, nbt_options) {
                Ok((_name, value)) => from_value::<IndexMap<String, NbtTagData>>(value)
                    .map_err(|e| debug!("Failed to parse tags NBT: {}", e))
                    .ok(),
                Err(e) => {
                    debug!("Failed to load tags NBT: {}", e);
                    None
                }
            }
        };

        let registries =
            if let (Some(registries_data), Some(tags_data)) = (registries_data, tags_data) {
                self.registry_keys
                    .iter()
                    .filter_map(|registry_key| {
                        match Registry::load_from_nbt(registry_key, &registries_data, &tags_data) {
                            Ok(registry) => Some((registry_key.clone(), registry)),
                            Err(err) => {
                                debug!(
                                    registry_key = ?registry_key,
                                    err = ?err,
                                    "Failed to load registry from NBT, skipping"
                                );
                                None
                            }
                        }
                    })
                    .collect()
            } else {
                HashMap::new()
            };

        RegistryManager { registries }
    }

    /// Register the default set of registry keys
    #[must_use]
    pub fn with_defaults(self) -> Self {
        self.register_all(RegistryKeys::ALL_REGISTRIES)
    }
}

impl Default for RegistryManagerBuilder {
    fn default() -> Self {
        Self::new()
    }
}
