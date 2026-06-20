use crate::cli::Commands;
use crate::convert::convert_data;
use crate::extract::{download_server_jar, run_data_generator};
use clap::Parser;
use protocol_version::protocol_version::ProtocolVersion;
use std::str::FromStr;
use tracing::{Level, info};
use tracing_subscriber::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

mod cli;
mod convert;
mod extract;
mod manifest;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = cli::Cli::parse();
    enable_logging(cli.verbose);

    match cli.command {
        Commands::Extract { version } => {
            let sanitized_version = sanitize_version(&version);
            let protocol_version = ProtocolVersion::from_str(&sanitized_version)?;
            let server_jar_path = download_server_jar(&protocol_version).await?;
            run_data_generator(&protocol_version, &server_jar_path).await?;
            info!("You may clean up the data and run the converter now: ");
            info!(
                "cargo run --package data_generator --bin data_generator -- convert {}",
                version
            );
        }
        Commands::Convert { version } => {
            convert_data(&version)?;
        }
    }

    Ok(())
}

fn sanitize_version(version: &str) -> String {
    if version.starts_with("V") {
        version.replace(".", "_")
    } else {
        format!("V{}", version.replace(".", "_"))
    }
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
