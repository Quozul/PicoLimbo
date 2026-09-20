use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct ModernForwardingConfig {
    enabled: bool,
    secret: String,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct BungeeCordForwardingConfig {
    enabled: bool,
    bungee_guard: bool,
    tokens: Vec<String>,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct StructuredForwarding {
    velocity: ModernForwardingConfig,
    bungee_cord: BungeeCordForwardingConfig,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(tag = "method", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TaggedForwarding {
    #[default]
    #[serde(alias = "none")]
    #[serde(alias = "disabled", alias = "off", alias = "DISABLED", alias = "OFF")]
    None,

    #[serde(alias = "legacy", alias = "bungee", alias = "BUNGEE", alias = "BUNGEECORD", alias = "BUNGECORD", alias = "BUNGEE_CORD", alias = "BUNGEE_LEGACY", alias = "LEGACY_FORWARDING")]
    Legacy,

    #[serde(alias = "bungee_guard")]
    BungeeGuard { tokens: Vec<String> },

    #[serde(alias = "modern", alias = "velocity", alias = "VELOCITY", alias = "VELOCITY_MODERN", alias = "MODERN_FORWARDING", alias = "VELOCITY_FORWARDING")]
    Modern { secret: String },
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum ForwardingConfig {
    Structured(StructuredForwarding),
    Tagged(TaggedForwarding),
}

impl Default for ForwardingConfig {
    fn default() -> Self {
        Self::Tagged(TaggedForwarding::default())
    }
}

impl From<ForwardingConfig> for TaggedForwarding {
    fn from(cfg: ForwardingConfig) -> Self {
        match cfg {
            ForwardingConfig::Tagged(forwarding) => forwarding,
            ForwardingConfig::Structured(forwarding) => {
                if forwarding.velocity.enabled {
                    Self::Modern {
                        secret: forwarding.velocity.secret,
                    }
                } else if forwarding.bungee_cord.enabled {
                    if forwarding.bungee_cord.bungee_guard {
                        Self::BungeeGuard {
                            tokens: forwarding.bungee_cord.tokens,
                        }
                    } else {
                        Self::Legacy
                    }
                } else {
                    Self::None
                }
            }
        }
    }
}
