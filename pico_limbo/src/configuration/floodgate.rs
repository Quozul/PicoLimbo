use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(default)]
#[serde(deny_unknown_fields)]
pub struct FloodgateConfig {
    /// Enable standard Geyser/Floodgate authentication data.
    pub enabled: bool,

    /// Path to the Floodgate key.pem file.
    pub key: String,

    /// Username prefix applied to standard Floodgate players.
    pub username_prefix: String,

    /// Replace spaces in Bedrock usernames with underscores.
    pub replace_spaces: bool,

    /// Education Edition / EduFloodgate settings.
    pub education: EducationFloodgateConfig,
}

#[derive(Serialize, Deserialize)]
#[serde(default)]
#[serde(deny_unknown_fields)]
pub struct EducationFloodgateConfig {
    /// Enable EduGeyser/EduFloodgate authentication data.
    pub enabled: bool,

    /// Username prefix applied to Education Edition players.
    pub username_prefix: String,

    /// Use the legacy Education Edition UUID format.
    pub uuid_legacy: bool,
}

impl Default for FloodgateConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            key: "key.pem".into(),
            username_prefix: ".".into(),
            replace_spaces: true,
            education: EducationFloodgateConfig::default(),
        }
    }
}

impl Default for EducationFloodgateConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            username_prefix: "+".into(),
            uuid_legacy: false,
        }
    }
}
