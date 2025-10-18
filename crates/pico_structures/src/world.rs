use crate::chunk_processor::{ChunkProcessor, ChunkProcessorError};
use crate::palette::Palette;
use crate::prelude::Schematic;
use minecraft_protocol::prelude::Coordinates;
use rayon::iter::ParallelIterator;
use rayon::prelude::IntoParallelIterator;
use thiserror::Error;

pub struct World {
    world_sections: Vec<Palette>,
    size_in_chunks: Coordinates,
    schematic: Schematic,
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

        Ok(Self {
            world_sections: world_sections?,
            size_in_chunks,
            schematic,
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

    pub fn get_schematic(&self) -> &Schematic {
        &self.schematic
    }
}
