mod cli;
#[cfg(feature = "server")]
mod configuration;
#[cfg(feature = "server")]
mod forwarding;
#[cfg(feature = "server")]
mod handlers;
#[cfg(feature = "ping_util")]
mod ping_util;
#[cfg(feature = "server")]
mod server;
#[cfg(feature = "server")]
mod server_state;

use crate::cli::{Cli, Commands};
use clap::Parser;
use std::process::ExitCode;
use tracing::Level;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();
    enable_logging(cli.verbose);

    match cli.command {
        #[cfg(feature = "ping_util")]
        Commands::Ping {
            address,
            json,
            version,
        } => ping_util::ping_server::parse_cli_for_ping(address, json, version).await,
        #[cfg(feature = "server")]
        Commands::Server {
            data_directory,
            config_path,
        } => server::start_server::start_server(data_directory, config_path).await,
    }
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
