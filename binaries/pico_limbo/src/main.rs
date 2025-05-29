mod cli;
mod config;
mod handlers;
mod server_state;
mod velocity;

use crate::cli::Cli;
use crate::config::Config;
use crate::handlers::handshake::on_handshake;
use crate::handlers::login::{on_custom_query_answer, on_login_acknowledged, on_login_start};
use crate::handlers::play::on_player_position;
use crate::handlers::status::{on_ping_request, on_status_request};
use crate::server_state::{ServerState, ServerStateBuildError};
use clap::Parser;
use minecraft_packets::play::Dimension;
use minecraft_server::server::Server;
use std::path::PathBuf;
use tracing::{Level, debug};
use tracing_subscriber::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    enable_logging(cli.verbose);
    let cfg = config::load_or_create(cli.config_path).expect("failed to create configuration file");
    let bind = cfg.bind.clone();

    let server_state =
        build_state(cli.data_directory, cfg).expect("Failed to initialize server state");

    Server::<ServerState>::new(bind, server_state)
        .on(on_handshake)
        .on(on_status_request)
        .on(on_ping_request)
        .on(on_login_start)
        .on(on_login_acknowledged)
        .on(on_custom_query_answer)
        .on(on_player_position)
        .run()
        .await;
}

fn enable_logging(verbose: u8) {
    let log_level = match verbose {
        0 => Level::INFO,
        1 => Level::DEBUG,
        2 => Level::TRACE,
        _ => Level::TRACE,
    };

    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env().add_directive(log_level.into()))
        .with(tracing_subscriber::fmt::layer().with_target(false))
        .init();
}

fn build_state(
    asset_directory: PathBuf,
    cfg: Config,
) -> Result<ServerState, ServerStateBuildError> {
    let mut server_state_builder = ServerState::builder();

    if cfg.secret_key.is_empty() {
        server_state_builder.modern_forwarding(false);
    } else {
        debug!("Enabling modern forwarding");
        server_state_builder
            .modern_forwarding(true)
            .secret_key(cfg.secret_key);
    }

    server_state_builder
        .data_directory(asset_directory)
        .dimension(Dimension::from_name(&cfg.spawn_dimension).unwrap_or_default())
        .description_text(&cfg.message_of_the_day)
        .max_players(cfg.max_players);

    server_state_builder.build()
}
