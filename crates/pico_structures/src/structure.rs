use crate::blocks_report::BlocksReports;
use crate::search_block_state::SearchState;
use minecraft_protocol::prelude::VarInt;
use minecraft_protocol::protocol_version::ProtocolVersion;
use nbt::prelude::{Nbt, NbtDecodeError};
use std::collections::{HashMap, HashSet};
use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum StructureError {
    #[error("Error while decoding NBT: {0}")]
    Nbt(#[from] NbtDecodeError),
}

pub struct Structure {
    palette: Vec<VarInt>,
    block_lookup: HashMap<(i32, i32, i32), i32>,
    dimensions: (i32, i32, i32),
    solid_blocks_count: usize,
    air: i32,
}

impl Structure {
    const AIR: i32 = 0;

    pub fn load_structure_file(
        path: &Path,
        version: ProtocolVersion,
    ) -> Result<Self, StructureError> {
        let blocks_reports = BlocksReports::new().unwrap_or_default();
        let structure_nbt = Nbt::from_file(path)?;

        let palette_nbt: Vec<Nbt> = structure_nbt
            .find_tag("palette")
            .unwrap()
            .get_vec()
            .unwrap();

        let palette: Vec<i32> = palette_nbt
            .iter()
            .map(|nbt| Self::get_block_id_from_nbt(nbt, version, &blocks_reports))
            .collect::<HashSet<i32>>()
            .into_iter()
            .collect();

        let block_id_to_palette_index: HashMap<i32, i32> = palette
            .iter()
            .enumerate()
            .map(|(index, &block_id)| (block_id, index as i32))
            .collect();

        let size = structure_nbt
            .find_tag("size")
            .unwrap()
            .get_vec()
            .unwrap()
            .iter()
            .map(|nbt| nbt.get_int().unwrap())
            .collect::<Vec<i32>>();
        let dimensions = (size[0], size[1], size[2]);

        let blocks: Vec<Nbt> = structure_nbt.find_tag("blocks").unwrap().get_vec().unwrap();
        let mut block_lookup = HashMap::with_capacity(blocks.len());
        let air = block_id_to_palette_index
            .get(&Self::AIR)
            .copied()
            .unwrap_or_default();

        for block in blocks {
            let pos = block.find_tag("pos").unwrap().get_vec().unwrap();
            let x = pos[0].get_int().unwrap();
            let y = pos[1].get_int().unwrap();
            let z = pos[2].get_int().unwrap();

            let palette_index = block.find_tag("state").unwrap().get_int().unwrap() as usize;
            let block_nbt = &palette_nbt[palette_index];
            let block_id = Self::get_block_id_from_nbt(block_nbt, version, &blocks_reports);

            let final_palette_index = block_id_to_palette_index
                .get(&block_id)
                .copied()
                .unwrap_or(air);

            block_lookup.insert((x, y, z), final_palette_index);
        }

        let solid_blocks_count = Self::count_non_air_blocks(&structure_nbt);

        Ok(Self {
            palette: palette.iter().map(VarInt::from).collect(),
            block_lookup,
            dimensions,
            solid_blocks_count,
            air,
        })
    }

    pub fn get_block_at(&self, x: i32, y: i32, z: i32) -> i32 {
        if self.is_out_of_bounds(x, y, z) {
            return self.air;
        }

        self.block_lookup
            .get(&(x, y, z))
            .copied()
            .unwrap_or(self.air)
    }

    pub fn get_palette(&self) -> &Vec<VarInt> {
        &self.palette
    }

    pub fn get_solid_block_count(&self) -> usize {
        self.solid_blocks_count
    }

    fn count_non_air_blocks(structure_nbt: &Nbt) -> usize {
        let palette_nbt: Vec<Nbt> = structure_nbt
            .find_tag("palette")
            .unwrap()
            .get_vec()
            .unwrap();
        let blocks: Vec<Nbt> = structure_nbt.find_tag("blocks").unwrap().get_vec().unwrap();

        blocks
            .iter()
            .filter(|block| {
                let palette_index = block.find_tag("state").unwrap().get_int().unwrap() as usize;

                let block_name = palette_nbt
                    .get(palette_index)
                    .map(|nbt| nbt.find_tag("Name").unwrap().get_string().unwrap())
                    .unwrap_or_default();

                !block_name.is_empty() && block_name != "minecraft:air"
            })
            .count()
    }

    fn is_out_of_bounds(&self, x: i32, y: i32, z: i32) -> bool {
        let (max_x, max_y, max_z) = self.dimensions;
        x >= max_x || y >= max_y || z >= max_z || x < 0 || y < 0 || z < 0
    }

    fn get_block_id_from_nbt(
        block: &Nbt,
        version: ProtocolVersion,
        blocks_reports: &BlocksReports,
    ) -> i32 {
        let block_name = block.find_tag("Name").unwrap().get_string().unwrap();
        let mut search_block_state = SearchState::new();
        search_block_state
            .version(version)
            .block_name(blocks_reports, block_name);
        if let Some(properties) = block.find_tag("Properties").map(|p| p.get_vec().unwrap()) {
            for property in properties {
                let property_name = property.get_name().unwrap();
                let property_value = property.get_string().unwrap();
                search_block_state.property(blocks_reports, property_name, property_value);
            }
        }
        search_block_state
            .find(blocks_reports)
            .map(|x| x as i32)
            .unwrap_or(Self::AIR)
    }
}
