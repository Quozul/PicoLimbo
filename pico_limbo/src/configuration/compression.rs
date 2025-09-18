use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(default)]
#[serde(deny_unknown_fields)]
pub struct CompressionConfig {
    pub threshold: i32,
    pub level: u32,
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            threshold: -1,
            level: 6,
        }
    }
}
