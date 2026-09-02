use pico_identifier::Identifier;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct RegistriesReport {
    #[serde(flatten)]
    pub registries: HashMap<Identifier, Registry>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Registry {
    // Both are only here because `deny_unknown_fields` requires every key of the
    // report to be modelled; nothing reads them yet.
    #[serde(default)]
    #[allow(dead_code)]
    pub default: Option<String>,
    pub entries: HashMap<Identifier, Entry>,
    #[allow(dead_code)]
    pub protocol_id: u32,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Entry {
    pub protocol_id: u32,
}

impl RegistriesReport {
    pub fn from_resource_path(resource_path: &Path) -> crate::Result<Self> {
        let registries_report_path = resource_path.join("reports").join("registries.json");
        let json_str = std::fs::read_to_string(&registries_report_path)?;
        Ok(serde_json::from_str(&json_str)?)
    }
}
