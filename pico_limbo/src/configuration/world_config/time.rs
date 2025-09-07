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
