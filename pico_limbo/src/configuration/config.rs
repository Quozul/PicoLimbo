use crate::configuration::boss_bar::BossBarConfig;
use crate::configuration::commands::CommandsConfig;
use crate::configuration::compression::CompressionConfig;
use crate::configuration::connection_config::ConnectionConfig;
use crate::configuration::env_placeholders::{EnvPlaceholderError, expand_env_placeholders};
use crate::configuration::floodgate::FloodgateConfig;
use crate::configuration::fly_config::FlyConfig;
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

#[derive(Serialize, Deserialize)]
#[serde(default)]
#[serde(deny_unknown_fields)]
#[allow(clippy::struct_excessive_bools)]
pub struct Config {
    pub bind: String,

    pub forwarding: ForwardingConfig,

    pub floodgate: FloodgateConfig,

    pub world: WorldConfig,

    pub server_list: ServerListConfig,

    pub connection: ConnectionConfig,

    pub welcome_message: String,

    pub action_bar: String,

    pub default_game_mode: GameModeConfig,

    pub hardcore: bool,

    pub compression: CompressionConfig,

    pub tab_list: TabListConfig,

    pub fetch_player_skins: bool,

    pub reduced_debug_info: bool,

    pub fly: FlyConfig,

    pub accept_transfers: bool,

    pub boss_bar: BossBarConfig,

    pub title: TitleConfig,

    pub commands: CommandsConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            bind: "0.0.0.0:25565".into(),
            server_list: ServerListConfig::default(),
            connection: ConnectionConfig::default(),
            welcome_message: "Welcome to PicoLimbo!".into(),
            action_bar: "Welcome to PicoLimbo!".into(),
            forwarding: ForwardingConfig::default(),
            floodgate: FloodgateConfig::default(),
            default_game_mode: GameModeConfig::default(),
            world: WorldConfig::default(),
            hardcore: false,
            tab_list: TabListConfig::default(),
            fetch_player_skins: false,
            reduced_debug_info: false,
            boss_bar: BossBarConfig::default(),
            compression: CompressionConfig::default(),
            title: TitleConfig::default(),
            fly: FlyConfig::default(),
            accept_transfers: false,
            commands: CommandsConfig::default(),
        }
    }
}
