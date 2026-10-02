use crate::configuration::require_boolean::{require_false, require_true};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum FloodgateConfig {
    Enabled(EnabledFloodgateConfig),
    Disabled(DisabledFloodgateConfig),
}

#[derive(Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct EnabledFloodgateConfig {
    #[serde(deserialize_with = "require_true")]
    pub enabled: bool,
    pub key_file: String,
    pub username_prefix: String,
    pub replace_spaces: bool,
    pub education: bool,
    pub education_username_prefix: String,
    pub education_uuid_legacy: bool,
}

impl Default for EnabledFloodgateConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            key_file: "key.pem".into(),
            username_prefix: ".".into(),
            replace_spaces: true,
            education: false,
            education_username_prefix: "+".into(),
            education_uuid_legacy: false,
        }
    }
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DisabledFloodgateConfig {
    #[serde(deserialize_with = "require_false")]
    pub enabled: bool,
}

impl Default for FloodgateConfig {
    fn default() -> Self {
        Self::Disabled(DisabledFloodgateConfig { enabled: false })
    }
}
