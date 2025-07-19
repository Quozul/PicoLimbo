use clap::{Parser, Subcommand};
#[cfg(feature = "server")]
use std::path::PathBuf;

#[derive(Parser, Clone)]
#[command(
    about = "A lightweight Minecraft server written in Rust supporting all Minecraft versions"
)]
pub struct Cli {
    /// Enable verbose logging
    #[arg(
        short = 'v',
        long = "verbose",
        action = clap::ArgAction::Count,
        help = "Enable verbose logging (-v for debug, -vv for trace)"
    )]
    pub verbose: u8,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Clone)]
pub enum Commands {
    /// Ping a server
    #[cfg(feature = "ping_util")]
    Ping {
        /// If provided, will ping the server at the given address
        address: String,

        /// If provided and set to true, will print the ServerStatusResponse in JSON format
        #[arg(short, long, default_value_t = false)]
        json: bool,

        /// If provided, will ping the server with the given protocol version,
        /// otherwise, it will use the latest version currently supported
        #[arg(short, long)]
        version: Option<String>,
    },
    /// Start limbo server
    #[cfg(feature = "server")]
    Server {
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
        data_directory: PathBuf,

        /// Path to the TOML configuration file
        #[arg(
            short = 'c',
            long = "config",
            value_name = "CONFIG_PATH",
            default_value = "server.toml",
            help = "Configuration file path"
        )]
        config_path: PathBuf,
    },
}
