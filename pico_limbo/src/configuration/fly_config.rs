use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FlyConfig {
    /// If set to true, allows players to fly.
    pub allow_flight: bool,

    /// If set to true, players start in a flying state.
    pub flying: bool,

    /// The initial flying speed for players.
    pub flying_speed: f32,
}

impl Default for FlyConfig {
    fn default() -> Self {
        Self {
            allow_flight: false,
            flying: false,
            flying_speed: 0.05,
        }
    }
}
