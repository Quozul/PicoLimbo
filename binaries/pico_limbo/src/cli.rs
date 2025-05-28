use clap::Parser;
use shadow_rs::shadow;

shadow!(build);

#[derive(Parser)]
#[command(
    version = build::CLAP_LONG_VERSION,
    about = "A lightweight Minecraft server written from scratch in Rust supporting Minecraft versions from 1.7.2 up to 1.21.5"
)]
pub struct Cli {
    /// Sets the listening address
    #[arg(short, long, default_value = "127.0.0.1:25565")]
    pub address: String,

    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub debug: u8,

    /// Sets the secret key to enable Velocity modern forwarding
    #[arg(short, long)]
    pub secret_key: Option<String>,
}
