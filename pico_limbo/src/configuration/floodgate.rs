use crate::configuration::require_boolean::{require_false, require_true};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum FloodgateConfig {
    Disabled(DisabledFloodgateConfig),
    Enabled(EnabledFloodgateConfig),
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EnabledFloodgateConfig {
    #[serde(deserialize_with = "require_true")]
    pub enabled: bool,
    #[serde(default = "default_key_file")]
    pub key_file: String,
    #[serde(default = "default_username_prefix")]
    pub username_prefix: String,
    #[serde(default = "default_replace_spaces")]
    pub replace_spaces: bool,
    #[serde(default)]
    pub education: bool,
    #[serde(default = "default_education_username_prefix")]
    pub education_username_prefix: String,
    #[serde(default)]
    pub education_uuid_legacy: bool,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DisabledFloodgateConfig {
    #[serde(deserialize_with = "require_false")]
    pub enabled: bool,
    #[serde(default = "default_key_file")]
    pub key_file: String,
    #[serde(default = "default_username_prefix")]
    pub username_prefix: String,
    #[serde(default = "default_replace_spaces")]
    pub replace_spaces: bool,
    #[serde(default)]
    pub education: bool,
    #[serde(default = "default_education_username_prefix")]
    pub education_username_prefix: String,
    #[serde(default)]
    pub education_uuid_legacy: bool,
}

fn default_key_file() -> String {
    "key.pem".to_string()
}

fn default_username_prefix() -> String {
    ".".to_string()
}

fn default_replace_spaces() -> bool {
    true
}

fn default_education_username_prefix() -> String {
    "+".to_string()
}

impl Default for FloodgateConfig {
    fn default() -> Self {
        Self::Disabled(DisabledFloodgateConfig {
            enabled: false,
            key_file: default_key_file(),
            username_prefix: default_username_prefix(),
            replace_spaces: default_replace_spaces(),
            education: false,
            education_username_prefix: default_education_username_prefix(),
            education_uuid_legacy: false,
        })
    }
}
