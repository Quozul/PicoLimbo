//! Schematic to SNBT converter example
//!
//! Reads a Schematic file (compressed or uncompressed) and prints it as SNBT.
//! Only supports Sponge V2 and V3 formats.
use clap::Parser;
use pico_nbt::from_path_struct;
use pico_structures::prelude::SchematicFile;
use std::path::PathBuf;

/// Command‑line interface.
#[derive(Parser, Debug)]
#[command(name = "schema")]
#[command(
    about = "Reads a Sponge Schematic file and prints it as SNBT",
    long_about = None
)]
struct Cli {
    /// Path to the schematic file
    #[arg(required = true)]
    input: PathBuf,

    #[arg(short, long, action = clap::ArgAction::SetTrue)]
    full: bool,
}

fn print_summary(schematic: &SchematicFile) {
    println!("Version: {} (sponge.3)", schematic.get_version());
    println!("Dimensions: {}", schematic.get_dimensions());
    println!("Block palette size: {}", schematic.get_block_palette_max());
    println!("Total block integers: {}", schematic.get_block_data().len());
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let (_, schematic) = from_path_struct::<SchematicFile>(&cli.input)?;

    if cli.full {
        println!("{schematic:#?}");
    } else {
        print_summary(&schematic);
    }

    Ok(())
}
