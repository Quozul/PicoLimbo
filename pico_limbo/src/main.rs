#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
mod cli;
mod configuration;
mod forwarding;
mod handlers;
mod kick_messages;
mod server;
mod server_state;

use crate::cli::Cli;
use clap::Parser;
use std::process::ExitCode;

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();
    server::start_server::start_server(cli.config_path, cli.verbose).await
}
