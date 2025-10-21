use std::collections::HashMap;

use crate::chunk_processor::{ChunkProcessor, ChunkProcessorError};
use crate::palette::Palette;
use crate::prelude::Schematic;
use crate::schematic::IntermediateBlockEntityData;
use minecraft_protocol::prelude::Coordinates;
use rayon::iter::ParallelIterator;
use rayon::prelude::IntoParallelIterator;
use thiserror::Error;

pub struct World {
    world_sections: Vec<Palette>,
    size_in_chunks: Coordinates,
    // Store block entities organized by chunk (chunk_x, chunk_z) for fast lookup
    block_entities_by_chunk: HashMap<(i32, i32), Vec<PrecomputedBlockEntity>>,
}

#[derive(Clone)]
pub struct PrecomputedBlockEntity {
    pub world_x: i32,
    pub world_y: i32,
    pub world_z: i32,
    pub block_entity_type: String,
    pub nbt: IntermediateBlockEntityData,
}

#[derive(Debug, Error)]
pub enum WorldLoadingError {
    #[error(transparent)]
    ChunkProcessor(#[from] ChunkProcessorError),
}

impl World {
    pub fn from_schematic(schematic: Schematic) -> Result<Self, WorldLoadingError> {
        let dimensions = schematic.get_dimensions();
        let size_in_chunks = (dimensions + 15) / 16;
        let chunk_count = size_in_chunks.x() * size_in_chunks.y() * size_in_chunks.z();

        let world_sections: Result<Vec<_>, _> = (0..chunk_count)
            .into_par_iter()
            .map(|i| {
                let chunk_x = i / (size_in_chunks.y() * size_in_chunks.z());
                let chunk_y = (i / size_in_chunks.z()) % size_in_chunks.y();
                let chunk_z = i % size_in_chunks.z();

                let section_position = Coordinates::new(chunk_x, chunk_y, chunk_z);

                let mut processor = ChunkProcessor::new();
                processor.process_section(&schematic, section_position)
            })
            .collect();

        // Pre-compute block entities organized by chunk
        let mut block_entities_by_chunk: HashMap<(i32, i32), Vec<PrecomputedBlockEntity>> =
            HashMap::new();

        for entity_data in schematic.get_block_entities() {
            let world_x = entity_data.position.x();
            let world_y = entity_data.position.y();
            let world_z = entity_data.position.z();

            // Calculate which chunk this entity belongs to
            let chunk_x = world_x / 16;
            let chunk_z = world_z / 16;

            block_entities_by_chunk
                .entry((chunk_x, chunk_z))
                .or_default()
                .push(PrecomputedBlockEntity {
                    world_x,
                    world_y,
                    world_z,
                    block_entity_type: entity_data.block_entity_type.clone(),
                    nbt: entity_data.nbt.clone(),
                });
        }

        Ok(Self {
            world_sections: world_sections?,
            size_in_chunks,
            block_entities_by_chunk,
        })
    }

    pub fn get_section(&self, chunk_coords: &Coordinates) -> Option<&Palette> {
        if chunk_coords.x() < 0
            || chunk_coords.x() >= self.size_in_chunks.x()
            || chunk_coords.y() < 0
            || chunk_coords.y() >= self.size_in_chunks.y()
            || chunk_coords.z() < 0
            || chunk_coords.z() >= self.size_in_chunks.z()
        {
            return None;
        }

        let index = chunk_coords.z()
            + (chunk_coords.y() * self.size_in_chunks.z())
            + (chunk_coords.x() * self.size_in_chunks.y() * self.size_in_chunks.z());

        self.world_sections.get(index as usize)
    }

    pub fn get_chunk_block_entities(
        &self,
        chunk_x: i32,
        chunk_z: i32,
    ) -> Option<&[PrecomputedBlockEntity]> {
        self.block_entities_by_chunk
            .get(&(chunk_x, chunk_z))
            .map(|v| v.as_slice())
    }
}
