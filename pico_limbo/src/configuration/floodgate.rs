use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(tag = "method", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FloodgateConfig {
    #[serde(alias = "disabled")]
    Disabled,

    #[serde(alias = "enabled")]
    Enabled {
        key_file: String,
        username_prefix: String,
        replace_spaces: bool,
        education: EducationConfig,
    },
}

impl Default for FloodgateConfig {
    fn default() -> Self {
        Self::Disabled
    }
}

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct EducationConfig {
    pub enabled: bool,
    pub username_prefix: String,
    pub uuid_legacy: bool,
}

impl Default for EducationConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            username_prefix: "+".into(),
            uuid_legacy: false,
        }
    }
}
