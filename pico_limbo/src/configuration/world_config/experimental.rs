use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExperimentalWorldConfig {
    pub view_distance: i32,
    pub schematic_file: String,

    /// Lock the world time to the value of `world.time`
    pub lock_time: bool,
}

impl Default for ExperimentalWorldConfig {
    fn default() -> Self {
        Self {
            view_distance: 2,
            schematic_file: String::new(),
            lock_time: false,
        }
    }
}
