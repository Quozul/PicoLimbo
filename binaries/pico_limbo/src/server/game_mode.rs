use crate::configuration::game_mode_config::GameModeConfig;

#[derive(Default, Clone, Debug, PartialEq, Copy)]
#[repr(u8)]
pub enum GameMode {
    Survival = 0,
    Creative = 1,
    Adventure = 2,
    #[default]
    Spectator = 3,
}

impl From<GameModeConfig> for GameMode {
    fn from(value: GameModeConfig) -> Self {
        match value {
            GameModeConfig::Survival => GameMode::Survival,
            GameModeConfig::Creative => GameMode::Creative,
            GameModeConfig::Adventure => GameMode::Adventure,
            GameModeConfig::Spectator => GameMode::Spectator,
        }
    }
}

impl GameMode {
    pub fn value(self) -> u8 {
        self as u8
    }
}
