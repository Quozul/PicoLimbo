use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum GameModeConfig {
    Survival,
    Creative,
    Adventure,
    #[default]
    Spectator,
}

impl From<GameModeConfig> for u8 {
    fn from(value: GameModeConfig) -> Self {
        match value {
            GameModeConfig::Survival => 0,
            GameModeConfig::Creative => 1,
            GameModeConfig::Adventure => 2,
            GameModeConfig::Spectator => 3,
        }
    }
}
