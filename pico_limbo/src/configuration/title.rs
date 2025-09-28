use crate::configuration::require_boolean::{require_false, require_true};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum TitleConfig {
    Enabled(EnabledTitleConfig),
    Disabled(DisabledTitleConfig),
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EnabledTitleConfig {
    #[serde(deserialize_with = "require_true")]
    enabled: bool,
    pub title: String,
    pub sub_title: String,
    pub fade_in: i32,
    pub stay: i32,
    pub fade_out: i32,
}

#[derive(Deserialize, Serialize)]
pub struct DisabledTitleConfig {
    #[serde(deserialize_with = "require_false")]
    enabled: bool,
}

impl Default for TitleConfig {
    fn default() -> Self {
        Self::Enabled(EnabledTitleConfig {
            enabled: false,
            title: "<bold>Welcome!</bold>".to_string(),
            sub_title: "Enjoy your stay".to_string(),
            fade_in: 10,
            stay: 70,
            fade_out: 20,
        })
    }
}
