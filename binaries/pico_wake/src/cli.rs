use clap::Parser;

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// Sets the listening address
    #[arg(short, long, default_value = "127.0.0.1:25565")]
    pub address: String,

    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub debug: u8,

    /// Address of the backend server
    #[arg(short, long, default_value = "127.0.0.1:25566")]
    pub backend: String,
}
