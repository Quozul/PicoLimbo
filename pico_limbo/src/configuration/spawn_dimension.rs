use minecraft_protocol::prelude::Dimension;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum SpawnDimensionConfig {
    Overworld,
    Nether,
    #[default]
    End,
}

impl From<SpawnDimensionConfig> for Dimension {
    fn from(dimension: SpawnDimensionConfig) -> Self {
        match dimension {
            SpawnDimensionConfig::Overworld => Self::Overworld,
            SpawnDimensionConfig::Nether => Self::Nether,
            SpawnDimensionConfig::End => Self::End,
        }
    }
}
