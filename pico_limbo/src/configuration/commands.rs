use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CommandsConfig {
    pub spawn: String,
    pub fly: String,
    pub fly_speed: String,
    pub transfer: String,
}

impl Default for CommandsConfig {
    fn default() -> Self {
        Self {
            spawn: "spawn".to_string(),
            fly: "fly".to_string(),
            fly_speed: "flyspeed".to_string(),
            transfer: "transfer".to_string(),
        }
    }
}
