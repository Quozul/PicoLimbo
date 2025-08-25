use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default)]
pub struct ExperimentalConfig {
    pub world: ExperimentalWorldConfig,
}

#[derive(Serialize, Deserialize)]
pub struct ExperimentalWorldConfig {
    pub spawn_position: (f64, f64, f64),
    pub view_distance: i32,
    pub schematic_file: String,
}

impl Default for ExperimentalWorldConfig {
    fn default() -> Self {
        Self {
            spawn_position: (0.0, 320.0, 0.0),
            view_distance: 2,
            schematic_file: String::new(),
        }
    }
}
