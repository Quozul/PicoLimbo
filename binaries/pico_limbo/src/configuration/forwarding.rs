use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ModernForwardingConfig {
    pub enabled: bool,
    pub secret: String,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct BungeeCordForwardingConfig {
    pub enabled: bool,
    pub bungee_guard: bool,
    pub tokens: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Forwarding {
    pub velocity: ModernForwardingConfig,
    pub bungee_cord: BungeeCordForwardingConfig,
}
