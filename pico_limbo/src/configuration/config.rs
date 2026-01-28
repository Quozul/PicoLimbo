use crate::configuration::boss_bar::BossBarConfig;
use crate::configuration::compression::CompressionConfig;
use crate::configuration::env_placeholders::{EnvPlaceholderError, expand_env_placeholders};
use crate::configuration::forwarding::ForwardingConfig;
use crate::configuration::game_mode_config::GameModeConfig;
use crate::configuration::server_list::ServerListConfig;
use crate::configuration::tab_list::TabListConfig;
use crate::configuration::title::TitleConfig;
use crate::configuration::world_config::WorldConfig;
use serde::{Deserialize, Serialize};
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

    #[error("Failed to apply environment placeholders: {0}")]
    EnvPlaceholder(#[from] EnvPlaceholderError),
}

/// Application configuration, serializable to/from TOML.
#[derive(Serialize, Deserialize)]
#[serde(default)]
#[serde(deny_unknown_fields)]
#[allow(clippy::struct_excessive_bools)]
pub struct Config {
    /// Server listening address and port.
    ///
    /// Specify the IP address and port the server should bind to.
    /// Use 0.0.0.0 to listen on all network interfaces.
    pub bind: String,

    pub forwarding: ForwardingConfig,

    pub world: WorldConfig,

    pub server_list: ServerListConfig,

    /// Message sent to the player after spawning in the world.
    pub welcome_message: String,

    pub action_bar: String,

    /// Sets the default game mode for players
    /// Valid values are: "survival", "creative", "adventure" or "spectator"
    pub default_game_mode: GameModeConfig,

    /// If set to true, will spawn the player in hardcode mode
    pub hardcore: bool,

    pub compression: CompressionConfig,

    pub tab_list: TabListConfig,

    pub fetch_player_skins: bool,

    pub reduced_debug_info: bool,

    pub allow_unsupported_versions: bool,

    pub allow_flight: bool,

    pub boss_bar: BossBarConfig,

    pub title: TitleConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            bind: "0.0.0.0:25565".into(),
            server_list: ServerListConfig::default(),
            welcome_message: "Welcome to PicoLimbo!".into(),
            action_bar: "Welcome to PicoLimbo!".into(),
            forwarding: ForwardingConfig::default(),
            default_game_mode: GameModeConfig::default(),
            world: WorldConfig::default(),
            hardcore: false,
            tab_list: TabListConfig::default(),
            fetch_player_skins: false,
            reduced_debug_info: false,
            boss_bar: BossBarConfig::default(),
            compression: CompressionConfig::default(),
            title: TitleConfig::default(),
            allow_unsupported_versions: false,
            allow_flight: false,
        }
    }
}

/// Loads a `Config` from the given path.
/// If the file does not exist, it will be created (parent dirs too)
/// and populated with default values.
pub fn load_or_create<P: AsRef<Path>>(path: P) -> Result<Config, ConfigError> {
    let path = path.as_ref();

    if path.exists() {
        let raw_toml_str = fs::read_to_string(path)?;

        if raw_toml_str.trim().is_empty() {
            create_default_config(path)
        } else {
            let expanded_toml_str = expand_env_placeholders(&raw_toml_str)?;
            let cfg: Config = toml::from_str(expanded_toml_str.as_ref())?;
            Ok(cfg)
        }
    } else {
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir)?;
        }
        create_default_config(path)
    }
}

fn create_default_config<P: AsRef<Path>>(path: P) -> Result<Config, ConfigError> {
    let cfg = Config::default();
    let toml_str = toml::to_string_pretty(&cfg)?;
    fs::write(path, toml_str)?;
    Ok(cfg)
}
