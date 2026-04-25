use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PickItemConfig {
    pub enabled: bool,
    pub stack_size: i32,
}

impl Default for PickItemConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            stack_size: 1,
        }
    }
}
