use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default)]
pub struct ModernForwardingConfig {
    pub enabled: bool,
    pub secret: String,
}

#[derive(Serialize, Deserialize, Default)]
pub struct BungeeCordForwardingConfig {
    pub enabled: bool,
    pub bungee_guard: bool,
    pub tokens: Vec<String>,
}

#[derive(Serialize, Deserialize, Default)]
pub struct ForwardingConfig {
    pub velocity: ModernForwardingConfig,
    pub bungee_cord: BungeeCordForwardingConfig,
}
