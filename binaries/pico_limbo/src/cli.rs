use clap::Parser;
use shadow_rs::shadow;
use std::path::PathBuf;

shadow!(build);

#[derive(Parser)]
#[command(
    version = build::CLAP_LONG_VERSION,
    about = "A lightweight Minecraft server written from scratch in Rust supporting Minecraft versions from 1.7.2 up to 1.21.5"
)]
pub struct Cli {
    /// Server listening address and port
    ///
    /// Specify the IP address and port the server should bind to.
    /// Use 0.0.0.0 to listen on all network interfaces.
    #[arg(
        short,
        long,
        default_value = "127.0.0.1:25565",
        value_name = "IP:PORT",
        help = "Server listening address and port"
    )]
    pub address: String,

    /// Enable verbose logging
    #[arg(
        short, 
        long, 
        action = clap::ArgAction::Count,
        help = "Enable verbose logging (-v for debug, -vv for trace)"
    )]
    pub verbose: u8,

    /// Velocity modern forwarding secret key
    ///
    /// When specified, enables Velocity modern forwarding using the provided
    /// secret key. This must match the secret configured in your Velocity
    /// proxy configuration. Leave unset to disable proxy support.
    #[arg(
        short,
        long,
        value_name = "KEY",
        help = "Secret key for Velocity modern forwarding (enables proxy support)"
    )]
    pub secret_key: Option<String>,

    /// Data directory path
    ///
    /// Path to the directory containing packet maps, registries, and other
    /// game data files required by the server.
    #[arg(
        short = 'd',
        long = "data-dir",
        value_name = "PATH",
        default_value = "./assets",
        help = "Directory containing packet maps and game registries"
    )]
    pub data_directory: PathBuf,
}
