use minecraft_packets::play::boss_bar_packet::{BossBarColor, BossBarDivision};
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
            Self::NoDivision => 0,
            Self::SixNotches => 6,
            Self::TenNotches => 10,
            Self::TwelveNotches => 12,
            Self::TwentyNotches => 20,
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
            0 => Ok(Self::NoDivision),
            6 => Ok(Self::SixNotches),
            10 => Ok(Self::TenNotches),
            12 => Ok(Self::TwelveNotches),
            20 => Ok(Self::TwentyNotches),
            _ => Err(Error::custom(format!(
                "Invalid value for BossBarDivision: {value}"
            ))),
        }
    }
}

impl From<BossBarColorConfig> for BossBarColor {
    fn from(value: BossBarColorConfig) -> Self {
        match value {
            BossBarColorConfig::Pink => Self::Pink,
            BossBarColorConfig::Blue => Self::Blue,
            BossBarColorConfig::Red => Self::Red,
            BossBarColorConfig::Green => Self::Green,
            BossBarColorConfig::Yellow => Self::Yellow,
            BossBarColorConfig::Purple => Self::Purple,
            BossBarColorConfig::White => Self::White,
        }
    }
}

impl From<BossBarDivisionConfig> for BossBarDivision {
    fn from(value: BossBarDivisionConfig) -> Self {
        match value {
            BossBarDivisionConfig::NoDivision => Self::NoDivision,
            BossBarDivisionConfig::SixNotches => Self::SixNotches,
            BossBarDivisionConfig::TenNotches => Self::TenNotches,
            BossBarDivisionConfig::TwelveNotches => Self::TwelveNotches,
            BossBarDivisionConfig::TwentyNotches => Self::TwentyNotches,
        }
    }
}
