mod cli;
mod handlers;
mod server_state;
mod velocity;

use crate::cli::Cli;
use crate::handlers::handshake::on_handshake;
use crate::handlers::login::{on_custom_query_answer, on_login_acknowledged, on_login_start};
use crate::handlers::play::on_player_position;
use crate::handlers::status::{on_ping_request, on_status_request};
use crate::server_state::{ServerState, ServerStateBuildError};
use clap::Parser;
use minecraft_server::server::Server;
use std::path::PathBuf;
use tracing::Level;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    enable_logging(cli.verbose);
    let server_state =
        build_state(cli.secret_key, cli.data_directory).expect("Failed to initialize server state");

    Server::<ServerState>::new(cli.address, server_state)
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
    secret_key: Option<String>,
    asset_directory: PathBuf,
) -> Result<ServerState, ServerStateBuildError> {
    let mut server_state_builder = ServerState::builder();

    let secret_key = secret_key.filter(|s| !s.is_empty());
    if let Some(secret_key) = secret_key {
        server_state_builder
            .modern_forwarding(true)
            .secret_key(secret_key);
    } else {
        server_state_builder.modern_forwarding(false);
    }

    server_state_builder.data_directory(asset_directory);

    server_state_builder.build()
}
