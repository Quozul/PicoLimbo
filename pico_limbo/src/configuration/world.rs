use crate::configuration::spawn_dimension::SpawnDimensionConfig;
use crate::server::packet_handler::PacketHandlerError;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct WorldConfig {
    /// Name of the dimension to spawn the player in.
    /// Supported: "overworld", "nether" or "end"
    pub spawn_dimension: SpawnDimensionConfig,

    /// Time of the world
    /// Supported: "sunrise", "noon", "sunset", "midnight" or ticks (0 - 24000)
    pub time_world: String,

    /// Lock the world time to the value of `time_world`
    pub lock_time: bool,
}

impl Default for WorldConfig {
    fn default() -> Self {
        Self {
            spawn_dimension: SpawnDimensionConfig::default(),
            time_world: "day".into(),
            lock_time: true,
        }
    }
}

pub trait ParseTime {
    fn parse_time(&self) -> Result<i64, PacketHandlerError>;
}

impl ParseTime for str {
    fn parse_time(&self) -> Result<i64, PacketHandlerError> {
        match self.to_lowercase().as_str() {
            "sunrise" | "day" => Ok(1000),
            "noon" => Ok(6000),
            "sunset" | "night" => Ok(13000),
            "midnight" => Ok(18000),
            other => {
                let ticks = i64::from_str(other).map_err(|_| {
                    PacketHandlerError::InvalidState(format!("Invalid time_world value: {other}"))
                })?;
                if (0..24000).contains(&ticks) {
                    Ok(ticks)
                } else {
                    Err(PacketHandlerError::InvalidState(format!(
                        "time_world ticks out of range (0–23999): {ticks}"
                    )))
                }
            }
        }
    }
}

impl ParseTime for String {
    fn parse_time(&self) -> Result<i64, PacketHandlerError> {
        self.as_str().parse_time()
    }
}
