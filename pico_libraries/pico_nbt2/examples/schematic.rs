//! Schematic to SNBT converter example
//!
//! Reads a Schematic file (compressed or uncompressed) and prints it as SNBT.
//! Only supports Sponge V2 and V3 formats.

use clap::Parser;
use pico_nbt2::{Value, from_path_struct};
use serde::de::Error;
use serde::{Deserialize, Deserializer};
use std::collections::HashMap;
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

#[derive(Deserialize, Debug)]
#[serde(untagged)]
#[allow(dead_code)]
enum SchematicFile {
    V3(SchematicV3Wrapper),
    V2(SchematicV2),
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
#[allow(dead_code)]
struct SchematicV3Wrapper {
    schematic: SchematicV3,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
#[allow(dead_code)]
struct SchematicV3 {
    version: i32,
    data_version: i32,
    #[serde(default)]
    metadata: Option<Metadata>,
    width: u16,
    height: u16,
    length: u16,
    #[serde(default)]
    offset: Option<[i32; 3]>,
    blocks: BlockContainer,
    #[serde(default)]
    biomes: Option<BiomeContainer>,
    #[serde(default)]
    entities: Option<Vec<Value>>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
#[allow(dead_code)]
struct BlockContainer {
    palette: HashMap<String, i32>,
    #[serde(deserialize_with = "deserialize_var_int_array")]
    data: Vec<i32>,
    #[serde(default)]
    block_entities: Option<Vec<Value>>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
#[allow(dead_code)]
struct BiomeContainer {
    palette: HashMap<String, i32>,
    #[serde(deserialize_with = "deserialize_var_int_array")]
    data: Vec<i32>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
#[allow(dead_code)]
struct SchematicV2 {
    version: i32,
    #[serde(default)]
    data_version: Option<i32>,
    #[serde(default)]
    metadata: Option<Metadata>,
    width: u16,
    height: u16,
    length: u16,
    #[serde(default)]
    offset: Option<[i32; 3]>,
    #[serde(default)]
    palette_max: i32,
    palette: HashMap<String, i32>,
    #[serde(alias = "BlockData", deserialize_with = "deserialize_var_int_array")]
    block_data: Vec<i32>,
    #[serde(alias = "TileEntities", default)]
    block_entities: Option<Vec<Value>>,
    #[serde(default)]
    entities: Option<Vec<Value>>,
    #[serde(default)]
    biome_palette_max: Option<i32>,
    #[serde(default)]
    biome_palette: Option<HashMap<String, i32>>,
    #[serde(
        alias = "BiomeData",
        deserialize_with = "deserialize_opt_var_int_array",
        default
    )]
    biome_data: Option<Vec<i32>>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
#[allow(dead_code)]
struct Metadata {
    name: Option<String>,
    author: Option<String>,
    date: Option<i64>,
    required_mods: Option<Vec<String>>,
}

fn deserialize_var_int_array<'de, D>(deserializer: D) -> Result<Vec<i32>, D::Error>
where
    D: Deserializer<'de>,
{
    let bytes: Vec<u8> = serde_bytes::deserialize(deserializer)?;
    let mut integers = Vec::new();
    let mut iter = bytes.into_iter();

    while iter.len() > 0 {
        let (mut value, mut shift) = (0, 0);
        loop {
            let byte = iter
                .next()
                .ok_or_else(|| Error::custom("var int truncated"))?;
            value |= i32::from(byte & 0x7F) << shift;
            if byte & 0x80 == 0 {
                break;
            }
            shift += 7;
            if shift >= 32 {
                return Err(Error::custom("var int too large"));
            }
        }
        integers.push(value);
    }
    Ok(integers)
}

fn deserialize_opt_var_int_array<'de, D>(deserializer: D) -> Result<Option<Vec<i32>>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    struct Wrapper(#[serde(deserialize_with = "deserialize_var_int_array")] Vec<i32>);

    let v = Option::<Wrapper>::deserialize(deserializer)?;
    Ok(v.map(|Wrapper(k)| k))
}

fn print_summary(schematic: &SchematicFile) {
    match schematic {
        SchematicFile::V3(SchematicV3Wrapper { schematic: v3 }) => {
            println!("Version: {} (sponge.3)", v3.version);
            println!("Dimensions: {}x{}x{}", v3.width, v3.height, v3.length);
            println!("Block palette size: {}", v3.blocks.palette.len());
            println!("Total block integers: {}", v3.blocks.data.len());
        }
        SchematicFile::V2(v2) => {
            println!("Version: {} (sponge.2)", v2.version);
            println!("Dimensions: {}x{}x{}", v2.width, v2.height, v2.length);
            println!("Block palette size: {}", v2.palette.len());
            println!("Total block integers: {}", v2.block_data.len());
        }
    }
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
