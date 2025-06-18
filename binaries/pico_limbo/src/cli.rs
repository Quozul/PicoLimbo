use clap::Parser;
use shadow_rs::shadow;
use std::path::PathBuf;

shadow!(build);

#[derive(Parser)]
#[command(
    version = build::CLAP_LONG_VERSION,
    about = "A lightweight Minecraft server written from scratch in Rust supporting Minecraft versions from 1.7.2 up to 1.21.6"
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
