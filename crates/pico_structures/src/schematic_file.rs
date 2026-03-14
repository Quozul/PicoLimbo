use minecraft_protocol::prelude::{Coordinates, InvalidCoordinateVec};
use pico_nbt::{Value, from_path_struct};
use serde::de::Error;
use serde::{Deserialize, Deserializer};
use std::collections::HashMap;
use std::path::Path;

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum SchematicFile {
    V3(SchematicV3Wrapper),
    V2(SchematicV2),
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct SchematicV3Wrapper {
    schematic: SchematicV3,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
#[allow(dead_code)]
pub struct SchematicV3 {
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
struct BlockContainer {
    palette: HashMap<String, i32>,
    #[serde(deserialize_with = "deserialize_var_int_array")]
    data: Vec<i32>,
    #[serde(default)]
    block_entities: Option<Vec<BlockEntity>>,
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
pub struct SchematicV2 {
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
    block_entities: Option<Vec<BlockEntity>>,
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

#[derive(Deserialize, Debug, Clone)]
pub struct BlockEntity {
    #[serde(rename = "Pos")]
    position: Vec<i32>,
    #[serde(rename = "Id")]
    identifier: String,
    #[serde(flatten)]
    data: Value,
}

impl BlockEntity {
    pub fn position(&self) -> Result<Coordinates, InvalidCoordinateVec> {
        Coordinates::try_from(self.position.clone())
    }

    pub fn identifier(&self) -> &str {
        self.identifier.as_str()
    }

    pub fn data(&self) -> &Value {
        &self.data
    }
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

impl SchematicFile {
    pub fn from_path(path: &Path) -> pico_nbt::Result<Self> {
        let (_, schematic) = from_path_struct::<SchematicFile>(path)?;
        Ok(schematic)
    }

    pub fn get_dimensions(&self) -> Coordinates {
        match self {
            SchematicFile::V3(SchematicV3Wrapper { schematic }) => Coordinates::new(
                i32::from(schematic.width),
                i32::from(schematic.height),
                i32::from(schematic.length),
            ),
            SchematicFile::V2(schematic) => Coordinates::new(
                i32::from(schematic.width),
                i32::from(schematic.height),
                i32::from(schematic.length),
            ),
        }
    }

    pub fn get_block_palette_max(&self) -> usize {
        match self {
            SchematicFile::V3(SchematicV3Wrapper { schematic }) => schematic.blocks.palette.len(),
            SchematicFile::V2(schematic) => usize::try_from(schematic.palette_max).unwrap_or(0),
        }
    }

    pub fn get_palette(&self) -> &HashMap<String, i32> {
        match self {
            SchematicFile::V3(SchematicV3Wrapper { schematic }) => &schematic.blocks.palette,
            SchematicFile::V2(schematic) => &schematic.palette,
        }
    }

    pub fn get_block_data(&self) -> &Vec<i32> {
        match self {
            SchematicFile::V3(SchematicV3Wrapper { schematic }) => &schematic.blocks.data,
            SchematicFile::V2(schematic) => &schematic.block_data,
        }
    }

    pub fn get_block_entities(&self) -> Option<&Vec<BlockEntity>> {
        match self {
            SchematicFile::V3(SchematicV3Wrapper { schematic }) => {
                schematic.blocks.block_entities.as_ref()
            }
            SchematicFile::V2(schematic) => schematic.block_entities.as_ref(),
        }
    }
}
