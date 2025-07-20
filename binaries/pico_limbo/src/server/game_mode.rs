use crate::configuration::game_mode_config::GameModeConfig;

#[derive(Default, Clone, Debug, PartialEq, Eq, Copy)]
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
            GameModeConfig::Survival => Self::Survival,
            GameModeConfig::Creative => Self::Creative,
            GameModeConfig::Adventure => Self::Adventure,
            GameModeConfig::Spectator => Self::Spectator,
        }
    }
}

impl GameMode {
    pub const fn value(self) -> u8 {
        self as u8
    }
}
