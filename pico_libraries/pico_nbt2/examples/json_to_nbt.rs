//! JSON to S/NBT converter example
//!
//! Reads a JSON file and convert it to NBT or prints it as SNBT.

use clap::{Parser, ValueEnum};
use pico_nbt2::{CompressionType, NbtOptions};
use serde_json::Value as JsonValue;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Input JSON file
    input: PathBuf,

    /// Output NBT file (optional, defaults to input filename with .nbt extension)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Enable nameless root tag (for network NBT)
    #[arg(long)]
    nameless_root: bool,

    /// Enable dynamic lists (heterogeneous lists)
    #[arg(long)]
    dynamic_lists: bool,

    /// Compression type
    #[arg(short, long, value_enum, default_value_t = Compression::Gzip)]
    compression: Compression,

    /// Print as SNBT instead of writing binary file
    #[arg(long)]
    snbt: bool,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Compression {
    None,
    Gzip,
    Zlib,
}

impl From<Compression> for CompressionType {
    fn from(c: Compression) -> Self {
        match c {
            Compression::None => Self::None,
            Compression::Gzip => Self::Gzip,
            Compression::Zlib => Self::Zlib,
        }
    }
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let file = File::open(&cli.input)?;
    let reader = BufReader::new(file);
    let json_data: JsonValue = serde_json::from_reader(reader)?;

    let nbt_value = pico_nbt2::json_to_nbt(json_data)?;
    let root_name = cli
        .input
        .file_stem()
        .unwrap_or_default()
        .to_str()
        .unwrap_or_default();

    if cli.snbt {
        if root_name.is_empty() {
            println!("{nbt_value:#?}");
        } else {
            println!("{root_name} {nbt_value:#?}");
        }
    } else {
        let output_path = cli.output.unwrap_or_else(|| {
            let mut p = cli.input.clone();
            p.set_extension("nbt");
            p
        });

        let file = File::create(&output_path)?;
        let writer = BufWriter::new(file);

        let mut encoder = pico_nbt2::encode(writer, cli.compression.into())?;

        let options = NbtOptions::new()
            .nameless_root(cli.nameless_root)
            .dynamic_lists(cli.dynamic_lists);

        pico_nbt2::to_writer_with_options(&mut encoder, &nbt_value, Some(root_name), options)?;

        println!(
            "Converted {} to {}",
            cli.input.display(),
            output_path.display()
        );
    }

    Ok(())
}
