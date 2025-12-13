use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct MetadataObject {
    name: Option<String>,
    author: Option<String>,
    data: Option<i64>,
    required_mods: Option<Vec<String>>,
}

type PaletteObject = HashMap<String, i32>;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct BlockEntityObject {
    pos: (i32, i32, i32),
    id: String,
    // extra: HashMap<String, Value<'a>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct EntityObject {
    pos: (i32, i32, i32),
    id: String,
    // extra: HashMap<String, Value<'a>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct SchematicV2 {
    version: i32,
    data_version: i32,
    metadata: MetadataObject,
    width: i16,
    height: i16,
    length: i16,
    offset: (i32, i32, i32),
    palette_max: i32,
    palette: PaletteObject,
    #[serde(with = "serde_bytes")]
    block_data: Vec<u8>,
    block_entities: Vec<BlockEntityObject>,
    entities: Option<Vec<EntityObject>>,
    biome_palette_max: Option<i32>,
    biome_palette: Option<PaletteObject>,
    // #[serde(with = "serde_bytes")]
    // biome_data: Option<Vec<u8>>,
}

fn main() {
    let path = PathBuf::from("data/schematics/spawn.schem");
    let schematic: SchematicV2 = pico_nbt2::from_path_struct(&path).expect("Failed to load nbt");
    println!("{schematic:#?}")
}
