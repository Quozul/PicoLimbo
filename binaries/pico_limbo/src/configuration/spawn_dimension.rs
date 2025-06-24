use minecraft_packets::play::Dimension;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Default)]
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
            SpawnDimensionConfig::Overworld => Dimension::Overworld,
            SpawnDimensionConfig::Nether => Dimension::Nether,
            SpawnDimensionConfig::End => Dimension::End,
        }
    }
}
