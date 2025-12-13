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
    #[serde(default)]
    pub default: Option<String>,
    pub entries: HashMap<Identifier, Entry>,
    pub protocol_id: u32,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Entry {
    pub protocol_id: u32,
}

impl RegistriesReport {
    pub fn from_resource_path(resource_path: &Path) -> Self {
        let registries_report_path = resource_path.join("reports").join("registries.json");
        let json_str = std::fs::read_to_string(&registries_report_path)
            .expect("failed to read registries.json");
        serde_json::from_str(&json_str).expect("failed to deserialize registries.json")
    }
}
