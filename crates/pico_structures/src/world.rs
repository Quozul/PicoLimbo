use crate::chunk_processor::{ChunkProcessor, ChunkProcessorError};
use crate::internal_block_entity::BlockEntity;
use crate::palette::Palette;
use crate::prelude::Schematic;
use minecraft_protocol::prelude::Coordinates;
use rayon::iter::ParallelIterator;
use rayon::prelude::IntoParallelIterator;
use thiserror::Error;

/// Light data for a single 16x16x16 chunk section.
/// Each byte contains two 4-bit light values (high nibble and low nibble).
pub type LightSection = Vec<i8>;

/// Alias for backward compatibility
pub type SkyLightSection = LightSection;

pub struct World {
    world_sections: Vec<Palette>,
    size_in_chunks: Coordinates,
    block_entities_by_chunk: Vec<Vec<BlockEntity>>,
    /// Sky light data indexed by chunk column (x, z), containing light for all Y sections
    sky_light_by_chunk: Vec<Vec<LightSection>>,
    /// Block light data indexed by chunk column (x, z), containing light for all Y sections
    block_light_by_chunk: Vec<Vec<LightSection>>,
}

#[derive(Debug, Error)]
pub enum WorldLoadingError {
    #[error(transparent)]
    ChunkProcessor(#[from] ChunkProcessorError),
}

impl World {
    pub fn from_schematic(schematic: &Schematic) -> Result<Self, WorldLoadingError> {
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
                processor.process_section(schematic, section_position)
            })
            .collect();

        let chunk_column_count = (size_in_chunks.x() * size_in_chunks.z()) as usize;
        let mut block_entities_by_chunk: Vec<Vec<BlockEntity>> =
            vec![Vec::new(); chunk_column_count];

        for entity_data in schematic.get_block_entities() {
            let world_x = entity_data.position.x();
            let world_z = entity_data.position.z();

            let chunk_x = world_x / 16;
            let chunk_z = world_z / 16;

            let index = (chunk_z + chunk_x * size_in_chunks.z()) as usize;

            if let Some(chunk_entities) = block_entities_by_chunk.get_mut(index) {
                chunk_entities.push(entity_data.clone());
            }
        }

        // Calculate sky light and block light globally across the entire schematic
        let (sky_light_by_chunk, block_light_by_chunk) =
            Self::calculate_global_light(schematic, size_in_chunks, chunk_column_count);

        Ok(Self {
            world_sections: world_sections?,
            size_in_chunks,
            block_entities_by_chunk,
        })
    }

    pub fn get_section(&self, chunk_coords: &Coordinates) -> Option<&Palette> {
        if !self.is_section_in_bounds(chunk_coords) {
            return None;
        }

        let index = chunk_coords.z()
            + (chunk_coords.y() * self.size_in_chunks.z())
            + (chunk_coords.x() * self.size_in_chunks.y() * self.size_in_chunks.z());

        self.world_sections.get(index as usize)
    }

    pub fn get_chunk_block_entities(&self, chunk_x: i32, chunk_z: i32) -> Option<&[BlockEntity]> {
        if !self.is_chunk_column_in_bounds(chunk_x, chunk_z) {
            return None;
        }

        let index = self.get_chunk_column_index(chunk_x, chunk_z);

        self.block_entities_by_chunk
            .get(index)
            .map(|v| v.as_slice())
    }

    /// Get pre-calculated sky light data for a chunk column.
    /// Returns a slice of LightSection, one for each Y section in the chunk.
    pub fn get_chunk_sky_light(&self, chunk_x: i32, chunk_z: i32) -> Option<&[LightSection]> {
        if !self.is_chunk_column_in_bounds(chunk_x, chunk_z) {
            return None;
        }

        let index = self.get_chunk_column_index(chunk_x, chunk_z);

        self.sky_light_by_chunk.get(index).map(|v| v.as_slice())
    }

    /// Get pre-calculated block light data for a chunk column.
    /// Returns a slice of LightSection, one for each Y section in the chunk.
    pub fn get_chunk_block_light(&self, chunk_x: i32, chunk_z: i32) -> Option<&[LightSection]> {
        if !self.is_chunk_column_in_bounds(chunk_x, chunk_z) {
            return None;
        }

        let index = self.get_chunk_column_index(chunk_x, chunk_z);

        self.block_light_by_chunk.get(index).map(|v| v.as_slice())
    }

    /// Get the number of Y sections in the world
    pub fn get_section_count_y(&self) -> i32 {
        self.size_in_chunks.y()
    }
}
