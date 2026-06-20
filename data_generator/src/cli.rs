use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Enable verbose logging
    #[arg(
        short = 'v',
        long = "verbose",
        action = clap::ArgAction::Count,
        help = "Enable verbose logging (-v for debug, -vv for trace)"
    )]
    pub verbose: u8,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Downloads the server jar for the specified protocol version and extracts the data.
    /// The server jars and all the extracted data are not versioned in the repository.
    /// If you wish to make edits to the registries and whatnot, you will have to run the extraction again.
    Extract { version: String },

    // TODO: Add an in-between command to convert the extracted data to a format that can be used by the convert command.
    // TODO: Add a command to do the reverse conversion, useful when cloning the repository and the user wish to make manual edits to the registries.
    /// This command assumes you have run the extract command beforehand and cleaned up the data in
    /// a compatible format for the conversion.
    /// It'll create a single NBT file containing all the registries for this particular version.
    Convert { version: String },
}
