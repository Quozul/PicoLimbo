use clap::Parser;
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

    /// Path to the TOML configuration file
    #[arg(
        short = 'c',
        long = "config",
        value_name = "CONFIG_PATH",
        default_value = "server.toml",
        help = "Configuration file path"
    )]
    pub config_path: PathBuf,
}
