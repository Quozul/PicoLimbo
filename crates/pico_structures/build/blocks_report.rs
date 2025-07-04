/*
{
  "minecraft:acacia_button": {
    "properties": {
      "face": [
        "floor",
        "wall",
        "ceiling"
      ],
      "facing": [
        "north",
        "south",
        "west",
        "east"
      ],
      "powered": [
        "true",
        "false"
      ]
    },
    "states": [
      {
        "properties": {
          "face": "wall",
          "facing": "north",
          "powered": "false"
        },
        "id": 5408,
        "default": true
      },
      {
        "properties": {
          "face": "wall",
          "facing": "south",
          "powered": "true"
        },
        "id": 5409
      }
    ]
  }
}
*/
use pico_codegen::prelude::{BinaryWriter, StringIndexer};
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use thiserror::Error;

#[derive(Deserialize, Debug)]
struct BlockState {
    #[serde(default)]
    default: bool,
    id: u32,
    #[serde(default)]
    properties: HashMap<String, String>,
}

#[derive(Deserialize, Debug)]
struct Block {
    #[serde(default)]
    properties: HashMap<String, Vec<String>>,
    #[serde(default)]
    states: Vec<BlockState>,
}

impl Block {
    pub fn get_default_id(&self) -> Option<u32> {
        self.states
            .iter()
            .find_map(|state| if state.default { Some(state.id) } else { None })
            .or(self.states.first().map(|state| state.id))
    }

    pub fn get_all_properties(&self) -> impl Iterator<Item = String> + '_ {
        self.properties.keys().cloned()
    }

    pub fn get_all_states(&self) -> impl Iterator<Item = String> + '_ {
        self.properties.values().flatten().cloned()
    }
}

#[derive(Deserialize, Debug)]
pub struct BlocksReport(HashMap<String, Block>);

#[derive(Error, Debug)]
pub enum BlocksReportError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
}

impl BlocksReport {
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<BlocksReport, BlocksReportError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        Ok(serde_json::from_reader(reader)?)
    }

    pub fn get_all_strings(&self) -> HashSet<String> {
        self.0
            .iter()
            .flat_map(|(name, block)| {
                std::iter::once(name.clone())
                    .chain(block.get_all_properties())
                    .chain(block.get_all_states())
            })
            .collect::<HashSet<_>>()
    }

    pub fn to_bytes(&self, indexer: &StringIndexer) -> Vec<u8> {
        let mut binary_writer = BinaryWriter::default();
        for (block_name, block) in &self.0 {
            binary_writer.write(indexer.get_index(block_name).unwrap());
            binary_writer.write(block.get_default_id().unwrap());
        }
        binary_writer.into_inner()
    }
}
