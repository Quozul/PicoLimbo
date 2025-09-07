use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default)]
pub struct ModernForwardingConfig {
    enabled: bool,
    secret: String,
}

#[derive(Serialize, Deserialize, Default)]
pub struct BungeeCordForwardingConfig {
    enabled: bool,
    bungee_guard: bool,
    tokens: Vec<String>,
}

#[derive(Serialize, Deserialize, Default)]
pub struct StructuredForwarding {
    velocity: ModernForwardingConfig,
    bungee_cord: BungeeCordForwardingConfig,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(tag = "method", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TaggedForwarding {
    #[default]
    #[serde(alias = "none")]
    None,

    #[serde(alias = "legacy")]
    Legacy,

    #[serde(alias = "bungee_guard")]
    BungeeGuard { tokens: Vec<String> },

    #[serde(alias = "modern")]
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
