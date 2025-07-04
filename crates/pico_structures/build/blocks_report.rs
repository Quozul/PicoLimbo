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

    pub fn get_state_count(&self) -> usize {
        self.states.len()
    }

    pub fn get_properties_count(&self) -> usize {
        self.properties.len()
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

    pub fn get_block_count(&self) -> u16 {
        self.0.len() as u16
    }

    pub fn to_bytes(&self, indexer: &StringIndexer) -> Vec<u8> {
        let mut writer = BinaryWriter::default();
        writer.write(self.get_block_count());
        for (block_name, block) in &self.0 {
            writer.write(indexer.get_index(block_name).unwrap());
            writer.write(block.get_default_id().unwrap());

            writer.write(block.get_properties_count() as u16);
            writer.write(block.get_state_count() as u16);

            for state in block.states.iter() {
                let property_ids =
                    state
                        .properties
                        .iter()
                        .map(|(property_name, property_value)| {
                            let name_id = indexer.get_index(property_name).unwrap();
                            let value_id = indexer.get_index(property_value).unwrap();
                            (name_id, value_id)
                        });
                for (name_id, value_id) in property_ids {
                    writer.write(name_id);
                    writer.write(value_id);
                }

                writer.write(state.id);
            }
        }
        writer.into_inner()
    }
}
