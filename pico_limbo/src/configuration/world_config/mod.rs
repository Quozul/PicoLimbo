use crate::configuration::world_config::boundaries::BoundariesConfig;
use crate::configuration::world_config::experimental::ExperimentalWorldConfig;
use crate::configuration::world_config::spawn_dimension::SpawnDimensionConfig;
use crate::configuration::world_config::time::TimeConfig;
use serde::{Deserialize, Serialize};

pub mod boundaries;
mod experimental;
mod spawn_dimension;
mod time;

#[derive(Serialize, Deserialize)]
#[serde(default)]
#[serde(deny_unknown_fields)]
pub struct WorldConfig {
    /// Position to spawn the players at
    pub spawn_position: (f64, f64, f64),

    /// Rotation to spawn the players at
    pub spawn_rotation: (f32, f32),

    /// Name of the dimension to spawn the player in.
    /// Supported: "overworld", "nether" or "end"
    pub dimension: SpawnDimensionConfig,

    /// Time of the world
    /// Supported: "sunrise", "noon", "sunset", "midnight" or ticks (0 - 24000)
    pub time: TimeConfig,

    /// Experimental settings
    pub experimental: ExperimentalWorldConfig,

    /// World Boundaries settings
    pub boundaries: BoundariesConfig,
}

impl Default for WorldConfig {
    fn default() -> Self {
        Self {
            spawn_position: (0.0, 320.0, 0.0),
            spawn_rotation: (0.0, 0.0),
            dimension: SpawnDimensionConfig::default(),
            time: TimeConfig::default(),
            experimental: ExperimentalWorldConfig::default(),
            boundaries: BoundariesConfig::default(),
        }
    }
}
