mod cli;
mod client;
mod client_context;
mod connection_handler;
mod handlers;
mod ping_server;
mod server;
mod server_manager;

use crate::cli::Cli;
use crate::client_context::ClientContext;
use crate::server::Server;
use clap::Parser;
use std::path::PathBuf;
use tracing::Level;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    enable_logging(cli.debug);

    let data_directory = std::env::var_os("DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("./assets"));
    let client_context = ClientContext::new(cli.backend, data_directory)?;
    Server::new(cli.address, client_context)
        .await?
        .run()
        .await?;

    Ok(())
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
