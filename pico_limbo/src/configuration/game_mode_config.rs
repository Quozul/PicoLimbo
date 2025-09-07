use crate::server::game_mode::GameMode;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum GameModeConfig {
    Survival,
    Creative,
    Adventure,
    #[default]
    Spectator,
}

impl From<GameModeConfig> for GameMode {
    fn from(value: GameModeConfig) -> Self {
        match value {
            GameModeConfig::Survival => Self::Survival,
            GameModeConfig::Creative => Self::Creative,
            GameModeConfig::Adventure => Self::Adventure,
            GameModeConfig::Spectator => Self::Spectator,
        }
    }
}
