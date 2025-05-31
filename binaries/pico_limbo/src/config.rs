use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::{fs, io};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("TOML serialization error: {0}")]
    TomlSerialize(#[from] toml::ser::Error),

    #[error("TOML deserialization error: {0}")]
    TomlDeserialize(#[from] toml::de::Error),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct ServerListConfig {
    /// Maximum amount of player displayed in the server list.
    pub max_players: u32,

    /// Description of the server displayed in the server list.
    pub message_of_the_day: String,

    /// Set to false to always show 0 online players
    pub show_online_player_count: bool,
}

impl Default for ServerListConfig {
    fn default() -> Self {
        Self {
            max_players: 20,
            message_of_the_day: "A Minecraft Server".into(),
            show_online_player_count: true,
        }
    }
}

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

/// Application configuration, serializable to/from TOML.
#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Server listening address and port.
    ///
    /// Specify the IP address and port the server should bind to.
    /// Use 0.0.0.0 to listen on all network interfaces.
    pub bind: String,

    pub forwarding: Forwarding,

    /// Name of the dimension to spawn the player in.
    /// Supported: "overworld", "nether" or "end"
    pub spawn_dimension: String,

    pub server_list: ServerListConfig,

    /// Message sent to the player after spawning in the world.
    pub welcome_message: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            bind: "0.0.0.0:25565".into(),
            spawn_dimension: "overworld".into(),
            server_list: ServerListConfig::default(),
            welcome_message: "Welcome to PicoLimbo!".into(),
            forwarding: Forwarding::default(),
        }
    }
}

/// Loads a `Config` from the given path.
/// If the file does not exist, it will be created (parent dirs too)
/// and populated with default values.
pub fn load_or_create<P: AsRef<Path>>(path: P) -> Result<Config, ConfigError> {
    let path = path.as_ref();

    if path.exists() {
        let toml_str = fs::read_to_string(path)?;
        let cfg = toml::from_str(&toml_str)?;
        Ok(cfg)
    } else {
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir)?;
        }

        let cfg = Config::default();
        let toml_str = toml::to_string_pretty(&cfg)?;

        let mut file = File::create(path)?;
        file.write_all(toml_str.as_bytes())?;
        file.flush()?;

        Ok(cfg)
    }
}
