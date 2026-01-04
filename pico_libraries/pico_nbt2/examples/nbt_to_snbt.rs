//! NBT to SNBT converter example
//!
//! Reads an NBT file (compressed or uncompressed) and prints it as SNBT.

use clap::Parser;
use pico_nbt2::from_path;
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Input NBT file
    input: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let (name, value) = from_path(&cli.input)?;

    if name.is_empty() {
        println!("{value:#?}");
    } else {
        println!("{name} {value:#?}");
    }

    Ok(())
}
