use minecraft_packets::play::boss_bar_packet::{BossBarColor, BossBarDivision};
use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Deserialize, Serialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum BossBarColorConfig {
    #[default]
    Pink,
    Blue,
    Red,
    Green,
    Yellow,
    Purple,
    White,
}

#[derive(Debug, Clone, Copy)]
pub struct BossBarDivisionSerde(pub BossBarDivision);

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BossBarConfig {
    pub enabled: bool,
    pub title: String,
    pub health: f32,
    pub color: BossBarColorConfig,
    pub division: BossBarDivisionSerde,
}

impl Default for BossBarConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            title: "<bold>Welcome to PicoLimbo!</bold>".to_string(),
            health: 1.0,
            color: BossBarColorConfig::Pink,
            division: BossBarDivisionSerde(BossBarDivision::NoDivision),
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

impl Serialize for BossBarDivisionSerde {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let value = match self.0 {
            BossBarDivision::NoDivision => 0,
            BossBarDivision::SixNotches => 6,
            BossBarDivision::TenNotches => 10,
            BossBarDivision::TwelveNotches => 12,
            BossBarDivision::TwentyNotches => 20,
        };
        serializer.serialize_u8(value)
    }
}

impl<'de> Deserialize<'de> for BossBarDivisionSerde {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = u8::deserialize(deserializer)?;
        let division = match value {
            0 => BossBarDivision::NoDivision,
            6 => BossBarDivision::SixNotches,
            10 => BossBarDivision::TenNotches,
            12 => BossBarDivision::TwelveNotches,
            20 => BossBarDivision::TwentyNotches,
            other => return Err(Error::custom(format!("invalid division value: {}", other))),
        };
        Ok(BossBarDivisionSerde(division))
    }
}
