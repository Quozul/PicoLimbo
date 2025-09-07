use crate::configuration::spawn_dimension::SpawnDimensionConfig;
use serde::{Deserialize, Deserializer, Serialize};
use std::str::FromStr;

#[derive(Serialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum TimeConfig {
    #[default]
    Day,
    Noon,
    Night,
    Midnight,
    Ticks(i64),
}

impl FromStr for TimeConfig {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "day" => Ok(Self::Day),
            "noon" => Ok(Self::Noon),
            "night" => Ok(Self::Night),
            "midnight" => Ok(Self::Midnight),
            _ => Err(format!("Invalid time config: {s}")),
        }
    }
}

impl<'de> Deserialize<'de> for TimeConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum TimeConfigHelper {
            String(String),
            Number(i64),
        }

        match TimeConfigHelper::deserialize(deserializer)? {
            TimeConfigHelper::String(s) => Self::from_str(&s).map_err(serde::de::Error::custom),
            TimeConfigHelper::Number(n) => Ok(Self::Ticks(n)),
        }
    }
}

impl From<TimeConfig> for i64 {
    fn from(t: TimeConfig) -> Self {
        match t {
            TimeConfig::Day => 1_000,
            TimeConfig::Noon => 6_000,
            TimeConfig::Night => 13_000,
            TimeConfig::Midnight => 18_000,
            TimeConfig::Ticks(ticks) => ticks,
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct WorldConfig {
    /// Name of the dimension to spawn the player in.
    /// Supported: "overworld", "nether" or "end"
    pub spawn_dimension: SpawnDimensionConfig,

    /// Time of the world
    /// Supported: "sunrise", "noon", "sunset", "midnight" or ticks (0 - 24000)
    pub time: TimeConfig,

    /// Lock the world time to the value of `time_world`
    pub lock_time: bool,
}

impl Default for WorldConfig {
    fn default() -> Self {
        Self {
            spawn_dimension: SpawnDimensionConfig::default(),
            time: TimeConfig::default(),
            lock_time: true,
        }
    }
}
