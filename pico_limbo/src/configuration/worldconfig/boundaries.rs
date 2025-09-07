use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(default)]
#[serde(deny_unknown_fields)]
pub struct BoundariesConfig {
    pub enabled: bool,

    pub min_y: i32,

    pub teleport_message: String,
}

impl Default for BoundariesConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            min_y: -64,
            teleport_message: "You have reached the bottom of the worldconfig.".into(),
        }
    }
}
