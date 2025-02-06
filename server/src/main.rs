mod cli;
mod client;
mod get_packet_length;
mod packet_error;
mod payload;
mod registry;
mod state;

use crate::cli::Cli;
use crate::client::Client;
use clap::Parser;
use tokio::net::TcpListener;
use tokio::time::{interval, Duration};
use tracing::{debug, error, info, Level};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    enable_logging(cli.debug);
    let listener = TcpListener::bind(&cli.address).await?;
    info!("listening on: {}", cli.address);

    while let Ok((inbound, address)) = listener.accept().await {
        debug!("accepted new client {}:{}", address.ip(), address.port());
        let mut client = Client::new(inbound);

        tokio::spawn(async move {
            let mut keep_alive_interval = interval(Duration::from_secs(20));

            loop {
                tokio::select! {
                    result = client.read_socket() => {
                        if let Err(err) = result {
                            error!("{err}");
                            return;
                        }

                        if client.is_complete() {
                            if let Err(err) = client
                                .handle()
                                .await
                                .and_then(|_| Ok(client.reset_payload()?))
                            {
                                error!("{err}");
                                break;
                            }
                        }
                    }
                    _ = keep_alive_interval.tick() => {
                        if let Err(err) = client.send_keep_alive().await {
                            error!("{err}");
                            break;
                        }
                    }
                }
            }
        });
    }

    Ok(())
}

pub fn enable_logging(verbose: u8) {
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
