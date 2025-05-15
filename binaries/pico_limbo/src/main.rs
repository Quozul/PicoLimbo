mod cli;
mod get_data_directory;
mod handlers;
mod velocity;

use crate::cli::Cli;
use crate::get_data_directory::get_data_directory;
use crate::handlers::handshake::on_handshake;
use crate::handlers::login::{on_custom_query_answer, on_login_acknowledged, on_login_start};
use crate::handlers::play::on_player_position;
use crate::handlers::status::{on_ping_request, on_status_request};
use clap::Parser;
use minecraft_server::server::Server;
use tracing::Level;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    enable_logging(cli.debug);

    let secret_key = cli.secret_key.filter(|s| !s.is_empty());
    let state = if let Some(secret_key) = secret_key {
        ServerState::modern_forwarding(secret_key.as_bytes())
    } else {
        ServerState::no_forwarding()
    };

    let data_directory = get_data_directory();
    Server::<ServerState>::new(cli.address, data_directory, state)
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

#[derive(Clone)]
struct ServerState {
    secret_key: Vec<u8>,
    modern_forwarding: bool,
}

impl ServerState {
    fn modern_forwarding(secret_key: &[u8]) -> Self {
        Self {
            secret_key: secret_key.to_vec(),
            modern_forwarding: true,
        }
    }

    fn no_forwarding() -> Self {
        Self {
            secret_key: Vec::new(),
            modern_forwarding: false,
        }
    }

    fn is_modern_forwarding(&self) -> bool {
        self.modern_forwarding
    }

    fn secret_key(&self) -> &[u8] {
        &self.secret_key
    }
}
