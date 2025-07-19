use crate::configuration::config::{Config, ConfigError, load_or_create};
use crate::handlers::configuration::on_acknowledge_finish_configuration;
use crate::handlers::handshake::on_handshake;
use crate::handlers::login::{on_custom_query_answer, on_login_acknowledged, on_login_start};
use crate::handlers::status::{on_ping_request, on_status_request};
use crate::server::network::Server;
use crate::server_state::{ServerState, ServerStateBuildError};
use minecraft_protocol::data::packets_report::packet_map::PacketMap;
use std::path::PathBuf;
use std::process::ExitCode;
use tracing::{debug, error};

pub async fn start_server(data_directory: PathBuf, config_path: PathBuf) -> ExitCode {
    let cfg = if let Some(cfg) = load_configuration(&config_path) {
        cfg
    } else {
        return ExitCode::FAILURE;
    };

    let bind = cfg.bind.clone();

    let server_state =
        build_state(&data_directory, cfg).expect("Failed to initialize server state");

    let packet_map = PacketMap::new(data_directory);

    Server::new(bind, server_state, packet_map)
        .on(on_handshake)
        .on(on_status_request)
        .on(on_ping_request)
        .on(on_login_start)
        .on(on_login_acknowledged)
        .on(on_custom_query_answer)
        .on(on_acknowledge_finish_configuration)
        .run()
        .await;

    ExitCode::SUCCESS
}

fn load_configuration(config_path: &PathBuf) -> Option<Config> {
    let cfg = load_or_create(config_path);
    match cfg {
        Err(ConfigError::TomlDeserialize(message, ..)) => {
            error!("Failed to load configuration: {}", message);
        }
        Err(ConfigError::Io(message, ..)) => {
            error!("Failed to load configuration: {}", message);
        }
        Err(ConfigError::TomlSerialize(message, ..)) => {
            error!("Failed to save default configuration file: {}", message);
        }
        Ok(cfg) => return Some(cfg),
    }
    None
}

fn build_state(
    asset_directory: &PathBuf,
    cfg: Config,
) -> Result<ServerState, ServerStateBuildError> {
    let mut server_state_builder = ServerState::builder();

    if cfg.forwarding.velocity.enabled {
        debug!("Enabling modern forwarding");
        server_state_builder.enable_modern_forwarding(cfg.forwarding.velocity.secret);
    } else if cfg.forwarding.bungee_cord.enabled {
        if cfg.forwarding.bungee_cord.bungee_guard {
            debug!("Enabling BungeeGuard forwarding");
            server_state_builder.enable_bungee_guard_forwarding(cfg.forwarding.bungee_cord.tokens);
        } else {
            debug!("Enabling legacy (BungeeCord) forwarding");
            server_state_builder.enable_legacy_forwarding();
        }
    } else {
        server_state_builder.disable_forwarding();
    }

    server_state_builder
        .data_directory(asset_directory)
        .dimension(cfg.spawn_dimension.into())
        .description_text(&cfg.server_list.message_of_the_day)
        .welcome_message(&cfg.welcome_message)
        .max_players(cfg.server_list.max_players)
        .show_online_player_count(cfg.server_list.show_online_player_count)
        .game_mode(cfg.default_game_mode.into());

    server_state_builder.build()
}
