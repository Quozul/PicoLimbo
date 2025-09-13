use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Deserialize, Serialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum BossBarColorConfig {
    #[default]
    Pink = 0,
    Blue = 1,
    Red = 2,
    Green = 3,
    Yellow = 4,
    Purple = 5,
    White = 6,
}

#[derive(Default)]
pub enum BossBarDivisionConfig {
    #[default]
    NoDivision,
    SixNotches,
    TenNotches,
    TwelveNotches,
    TwentyNotches,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BossBarConfig {
    pub enabled: bool,
    pub title: String,
    pub health: f32,
    pub color: BossBarColorConfig,
    pub division: BossBarDivisionConfig,
}

impl Default for BossBarConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            title: "<bold>Welcome to PicoLimbo!</bold>".to_string(),
            health: 1.0,
            color: BossBarColorConfig::Pink,
            division: BossBarDivisionConfig::NoDivision,
        }
    }
}

impl Serialize for BossBarDivisionConfig {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let value = match self {
            BossBarDivisionConfig::NoDivision => 0,
            BossBarDivisionConfig::SixNotches => 6,
            BossBarDivisionConfig::TenNotches => 10,
            BossBarDivisionConfig::TwelveNotches => 12,
            BossBarDivisionConfig::TwentyNotches => 20,
        };
        serializer.serialize_u8(value)
    }
}

impl<'de> Deserialize<'de> for BossBarDivisionConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = u8::deserialize(deserializer)?;
        match value {
            0 => Ok(BossBarDivisionConfig::NoDivision),
            6 => Ok(BossBarDivisionConfig::SixNotches),
            10 => Ok(BossBarDivisionConfig::TenNotches),
            12 => Ok(BossBarDivisionConfig::TwelveNotches),
            20 => Ok(BossBarDivisionConfig::TwentyNotches),
            _ => Err(Error::custom(format!(
                "Invalid value for BossBarDivision: {}",
                value
            ))),
        }
    }
}
