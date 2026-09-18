use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct FloodgateConfig {
    pub enabled: bool,
    pub key_file: String,
    pub username_prefix: String,
    pub replace_spaces: bool,
    pub education: bool,
}

impl Default for FloodgateConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            key_file: "key.pem".into(),
            username_prefix: ".".into(),
            replace_spaces: true,
            education: false,
        }
    }
}
