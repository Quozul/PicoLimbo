use nbt::prelude::{Nbt, NbtDecodeError};
use std::collections::HashSet;
use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum StructureError {
    #[error("Error while decoding NBT: {0}")]
    Nbt(#[from] NbtDecodeError),
}

#[derive(Debug)]
pub struct Structure {
    structure_nbt: Nbt,
    palette: Vec<i32>,
}

impl Structure {
    pub fn from_structure_file(path: &Path) -> Result<Self, StructureError> {
        let structure_nbt = Nbt::from_file(path)?;
        let palette: Vec<i32> = structure_nbt
            .find_tag("palette")
            .unwrap()
            .get_vec()
            .unwrap()
            .iter()
            .map(Self::get_block_id_from_nbt)
            .collect::<HashSet<i32>>()
            .into_iter()
            .collect();
        Ok(Self {
            structure_nbt,
            palette,
        })
    }

    pub fn get_block_at(&self, x: i32, y: i32, z: i32) -> i32 {
        if self.is_out_of_bounds(x, y, z) {
            return self
                .get_index_in_palette(Self::get_air())
                .unwrap_or_default();
        }

        let palette: Vec<Nbt> = self
            .structure_nbt
            .find_tag("palette")
            .unwrap()
            .get_vec()
            .unwrap();
        let blocks: Vec<Nbt> = self
            .structure_nbt
            .find_tag("blocks")
            .unwrap()
            .get_vec()
            .unwrap();
        let palette_index = blocks
            .iter()
            .find(|block| {
                let pos = block.find_tag("pos").unwrap().get_vec().unwrap();
                let block_x = pos.first().unwrap().get_int().unwrap();
                let block_y = pos[1].get_int().unwrap();
                let block_z = pos[2].get_int().unwrap();
                block_x == x && block_y == y && block_z == z
            })
            .map(|block| block.find_tag("state").unwrap().get_int().unwrap())
            .unwrap_or(0);

        let block = palette.get(palette_index as usize).unwrap();
        self.get_index_in_palette(Self::get_block_id_from_nbt(block))
            .unwrap_or_default()
    }

    pub fn get_palette(&self) -> Vec<i32> {
        self.palette.clone().into_iter().collect()
    }

    pub fn count_non_air_blocks(&self) -> usize {
        let palette: Vec<Nbt> = self
            .structure_nbt
            .find_tag("palette")
            .unwrap()
            .get_vec()
            .unwrap();
        let blocks: Vec<Nbt> = self
            .structure_nbt
            .find_tag("blocks")
            .unwrap()
            .get_vec()
            .unwrap();

        blocks
            .iter()
            .filter(|block| {
                let palette_index = block.find_tag("state").unwrap().get_int().unwrap() as usize;

                let block_name = palette
                    .get(palette_index)
                    .map(|nbt| nbt.find_tag("Name").unwrap().get_string().unwrap())
                    .unwrap_or_default();

                !block_name.is_empty() && block_name != "minecraft:air"
            })
            .count()
    }

    fn get_index_in_palette(&self, block_id: i32) -> Option<i32> {
        for (index, id) in self.palette.iter().enumerate() {
            if block_id == *id {
                return Some(index as i32);
            }
        }
        None
    }

    fn is_out_of_bounds(&self, x: i32, y: i32, z: i32) -> bool {
        let size = self
            .structure_nbt
            .find_tag("size")
            .unwrap()
            .get_vec()
            .unwrap()
            .iter()
            .map(|nbt| nbt.get_int().unwrap())
            .collect::<Vec<i32>>();
        let max_x = size[0];
        let max_y = size[1];
        let max_z = size[2];
        x >= max_x || y >= max_y || z >= max_z
    }

    fn get_air() -> i32 {
        SearchState::new()
            .block_name("minecraft:air")
            .build()
            .unwrap_or_default()
    }

    fn get_block_id_from_nbt(block: &Nbt) -> i32 {
        let block_name = block.find_tag("Name").unwrap().get_string().unwrap();
        let mut search_block_state = SearchState::new();
        search_block_state.block_name(block_name);
        if let Some(properties) = block.find_tag("Properties").map(|p| p.get_vec().unwrap()) {
            for property in properties {
                let property_name = property.get_name().unwrap();
                let property_value = property.get_string().unwrap();
                search_block_state.property(property_name, property_value);
            }
        }
        search_block_state.build().unwrap_or(Self::get_air())
    }
}

include!(concat!(env!("OUT_DIR"), "/blocks.rs"));
