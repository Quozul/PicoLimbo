use crate::configuration::TaggedForwarding;
use crate::configuration::boss_bar::BossBarConfig;
use crate::configuration::config::{Config, ConfigError, load_or_create};
use crate::configuration::tab_list::TabListMode;
use crate::configuration::title::TitleConfig;
use crate::configuration::world_config::boundaries::BoundariesConfig;
use crate::server::network::Server;
use crate::server_state::{ServerState, ServerStateBuilderError};
use std::path::PathBuf;
use std::process::ExitCode;
use tracing::{Level, debug, error};
use tracing_subscriber::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

pub async fn start_server(config_path: PathBuf, logging_level: u8) -> ExitCode {
    enable_logging(logging_level);
    let Some(cfg) = load_configuration(&config_path) else {
        return ExitCode::FAILURE;
    };

    let bind = cfg.bind.clone();

    match build_state(cfg) {
        Ok(server_state) => {
            Server::new(&bind, server_state).run().await;
            ExitCode::SUCCESS
        }
        Err(err) => {
            error!("Failed to start PicoLimbo: {err}");
            ExitCode::SUCCESS
        }
    }
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
        Err(ConfigError::EnvPlaceholder(var)) => {
            error!("Failed to load configuration: {}", var);
        }
        Err(ConfigError::TomlSerialize(message, ..)) => {
            error!("Failed to save default configuration file: {}", message);
        }
        Ok(cfg) => return Some(cfg),
    }
    None
}

fn build_state(cfg: Config) -> Result<ServerState, ServerStateBuilderError> {
    let mut server_state_builder = ServerState::builder();

    let forwarding: TaggedForwarding = cfg.forwarding.into();

    match forwarding {
        TaggedForwarding::None => {
            server_state_builder.disable_forwarding();
        }
        TaggedForwarding::Legacy => {
            debug!("Enabling legacy forwarding");
            server_state_builder.enable_legacy_forwarding();
        }
        TaggedForwarding::BungeeGuard { tokens } => {
            server_state_builder.enable_bungee_guard_forwarding(tokens);
        }
        TaggedForwarding::Modern { secret } => {
            debug!("Enabling modern forwarding");
            server_state_builder.enable_modern_forwarding(secret);
        }
    }

    if let BoundariesConfig::Enabled(boundaries) = cfg.world.boundaries {
        if cfg.world.spawn_position.1 < f64::from(boundaries.min_y) {
            return Err(ServerStateBuilderError::InvalidSpawnPosition);
        }
        server_state_builder.boundaries(boundaries.min_y, boundaries.teleport_message)?;
    }

    if let TabListMode::Enabled(ref tab_list) = cfg.tab_list.mode {
        server_state_builder.tab_list(&tab_list.header, &tab_list.footer)?;
    }

    if let BossBarConfig::Enabled(boss_bar) = cfg.boss_bar {
        server_state_builder.boss_bar(boss_bar)?;
    }

    if let TitleConfig::Enabled(title) = cfg.title {
        server_state_builder.title(
            &title.title,
            &title.subtitle,
            title.fade_in,
            title.stay,
            title.fade_out,
        )?;
    }

    let server_icon = cfg.server_list.server_icon;
    if std::fs::exists(&server_icon)? {
        server_state_builder.fav_icon(server_icon)?;
    }

    server_state_builder
        .dimension(cfg.world.dimension.into())
        .time_world(cfg.world.time.into())
        .lock_time(cfg.world.experimental.lock_time)
        .description_text(&cfg.server_list.message_of_the_day)
        .welcome_message(&cfg.welcome_message)
        .action_bar(&cfg.action_bar)?
        .max_players(cfg.server_list.max_players)
        .show_online_player_count(cfg.server_list.show_online_player_count)
        .game_mode(cfg.default_game_mode.into())
        .hardcore(cfg.hardcore)
        .spawn_position(cfg.world.spawn_position)
        .spawn_rotation(cfg.world.spawn_rotation)
        .view_distance(cfg.world.experimental.view_distance)
        .schematic(cfg.world.experimental.schematic_file)
        .enable_compression(cfg.compression.threshold, cfg.compression.level)?
        .fetch_player_skins(cfg.fetch_player_skins)
        .reduced_debug_info(cfg.reduced_debug_info)
        .set_player_listed(cfg.tab_list.player_listed)
        .set_reply_to_status(cfg.server_list.reply_to_status)
        .set_allow_unsupported_versions(cfg.allow_unsupported_versions)
        .set_allow_flight(cfg.allow_flight)
        .set_accept_transfers(cfg.accept_transfers)
        .server_commands(cfg.commands);

    server_state_builder.build()
}

fn enable_logging(verbose: u8) {
    let log_level = match verbose {
        0 => Level::INFO,
        1 => Level::DEBUG,
        _ => Level::TRACE,
    };

    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env().add_directive(log_level.into()))
        .with(tracing_subscriber::fmt::layer().with_target(false))
        .init();
}
