use nbt::prelude::{Nbt, NbtDecodeError};
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
}

impl Structure {
    pub fn from_structure_file(path: &Path) -> Result<Self, StructureError> {
        let structure_nbt = Nbt::from_file(path)?;
        Ok(Self { structure_nbt })
    }

    pub fn get_block_at(&self, x: i32, y: i32, z: i32) -> i32 {
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

        let block_name = palette
            .get(palette_index as usize)
            .map(|nbt| nbt.find_tag("Name").unwrap().get_string().unwrap())
            .unwrap_or_default();

        get_block_id(&block_name).unwrap_or(0)
    }
}

fn get_block_id(block_name: &str) -> Option<i32> {
    match block_name {
        "minecraft:air" => Some(0),
        "minecraft:cobblestone" => Some(12),
        "minecraft:oak_planks" => Some(13),
        "minecraft:oak_door" => Some(207),
        "minecraft:wall_torch" => Some(182),
        "minecraft:oak_stairs" => Some(187),
        "minecraft:glass_pane" => Some(328),
        "minecraft:white_bed" => Some(110),
        "minecraft:stripped_oak_log" => Some(68),
        "minecraft:jigsaw" => Some(866),
        _ => None,
    }
}
