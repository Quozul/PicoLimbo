use pico_identifier::Identifier;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Deserialize)]
#[allow(dead_code)] // TODO: This is currently unused, but should become used later on
pub struct RegistriesReport {
    #[serde(flatten)]
    pub registries: HashMap<Identifier, Registry>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
#[allow(dead_code)] // TODO: This is currently unused, but should become used later on
pub struct Registry {
    #[serde(default)]
    pub default: Option<String>,
    pub entries: HashMap<Identifier, Entry>,
    pub protocol_id: u32,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
#[allow(dead_code)] // TODO: This is currently unused, but should become used later on
pub struct Entry {
    pub protocol_id: u32,
}

impl RegistriesReport {
    #[allow(dead_code)] // TODO: This is currently unused, but should become used later on
    pub fn from_resource_path(resource_path: &Path) -> crate::Result<Self> {
        let registries_report_path = resource_path.join("reports").join("registries.json");
        let json_str = std::fs::read_to_string(&registries_report_path)?;
        Ok(serde_json::from_str(&json_str)?)
    }
}
