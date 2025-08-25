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
