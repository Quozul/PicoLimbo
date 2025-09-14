use crate::configuration::require_boolean::{require_false, require_true};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum TabListConfig {
    Enabled(EnabledTabListConfig),
    Disabled(DisabledTabListConfig),
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EnabledTabListConfig {
    #[serde(deserialize_with = "require_true")]
    enabled: bool,
    pub header: String,
    pub footer: String,
}

#[derive(Serialize, Deserialize)]
pub struct DisabledTabListConfig {
    #[serde(deserialize_with = "require_false")]
    enabled: bool,
}

impl Default for TabListConfig {
    fn default() -> Self {
        Self::Enabled(EnabledTabListConfig {
            enabled: true,
            header: "<bold>Welcome to PicoLimbo</bold>".to_string(),
            footer: "<green>Enjoy your stay!</green>".to_string(),
        })
    }
}
