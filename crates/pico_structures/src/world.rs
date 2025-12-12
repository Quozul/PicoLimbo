use crate::chunk_processor::{ChunkProcessor, ChunkProcessorError};
use crate::internal_block_entity::BlockEntity;
use crate::palette::Palette;
use crate::prelude::Schematic;
use minecraft_protocol::prelude::Coordinates;
use rayon::iter::ParallelIterator;
use rayon::prelude::{IntoParallelIterator, IntoParallelRefIterator};
use std::sync::atomic::{AtomicU8, Ordering};
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
            sky_light_by_chunk,
            block_light_by_chunk,
        })
    }

    /// Calculate both sky light and block light for the entire schematic at once.
    /// Returns (sky_light_by_chunk, block_light_by_chunk)
    fn calculate_global_light(
        schematic: &Schematic,
        size_in_chunks: Coordinates,
        chunk_column_count: usize,
    ) -> (Vec<Vec<LightSection>>, Vec<Vec<LightSection>>) {
        let dimensions = schematic.get_dimensions();
        let world_width = dimensions.x().max(size_in_chunks.x() * 16);
        let world_length = dimensions.z().max(size_in_chunks.z() * 16);
        let total_height = size_in_chunks.y() * 16;

        let volume_size = (total_height * world_width * world_length) as usize;

        // Pre-compute transparency as u8 (better cache performance than bool)
        let transparent: Vec<u8> = (0..volume_size)
            .into_par_iter()
            .map(|i| {
                let x = (i as i32) % world_width;
                let z = ((i as i32) / world_width) % world_length;
                let y = (i as i32) / (world_width * world_length);
                schematic.is_transparent(Coordinates::new(x, y, z)) as u8
            })
            .collect();

        // Initialize light volumes in parallel by columns
        let sky_light_atomic: Vec<AtomicU8> = (0..volume_size).map(|_| AtomicU8::new(0)).collect();
        let block_light_atomic: Vec<AtomicU8> = (0..volume_size).map(|_| AtomicU8::new(0)).collect();

        let column_count = (world_width * world_length) as usize;
        (0..column_count).into_par_iter().for_each(|col_idx| {
            let x = (col_idx as i32) % world_width;
            let z = (col_idx as i32) / world_width;
            let mut sky_light = 15u8;
            for y in (0..total_height).rev() {
                let idx = Self::get_light_index(x, y, z, world_width, world_length);

                if transparent[idx] != 0 {
                    sky_light_atomic[idx].store(sky_light, Ordering::Relaxed);
                } else {
                    sky_light = 0;
                }

                let pos = Coordinates::new(x, y, z);
                let emitted = schematic.get_emitted_light(pos);
                if emitted > 0 {
                    block_light_atomic[idx].store(emitted, Ordering::Relaxed);
                }
            }
        });

        let mut sky_light_volume: Vec<u8> = sky_light_atomic.into_iter().map(|a| a.into_inner()).collect();
        let mut block_light_volume: Vec<u8> = block_light_atomic.into_iter().map(|a| a.into_inner()).collect();

        // Propagate both light types in parallel using level-synchronized BFS
        Self::propagate_light_parallel(&mut sky_light_volume, &mut block_light_volume, &transparent, world_width, world_length, total_height);

        let sky_light_by_chunk = Self::convert_to_chunk_sections(
            &sky_light_volume, size_in_chunks, chunk_column_count, world_width, world_length, total_height, 15
        );
        let block_light_by_chunk = Self::convert_to_chunk_sections(
            &block_light_volume, size_in_chunks, chunk_column_count, world_width, world_length, total_height, 0
        );

        (sky_light_by_chunk, block_light_by_chunk)
    }

    #[inline]
    fn get_light_index(x: i32, y: i32, z: i32, world_width: i32, world_length: i32) -> usize {
        (y * world_width * world_length + z * world_width + x) as usize
    }

    /// Propagate both sky and block light using parallel level-synchronized BFS
    fn propagate_light_parallel(
        sky_light: &mut [u8],
        block_light: &mut [u8],
        transparent: &[u8],
        world_width: i32,
        world_length: i32,
        total_height: i32,
    ) {
        // Pre-compute neighbor offsets
        let stride_x = 1i32;
        let stride_z = world_width;
        let stride_y = world_width * world_length;
        let offsets = [-stride_x, stride_x, -stride_y, stride_y, -stride_z, stride_z];

        // Collect initial light sources for both types
        let mut sky_current: Vec<usize> = sky_light
            .iter()
            .enumerate()
            .filter(|&(_, &light)| light > 1)
            .map(|(i, _)| i)
            .collect();

        let mut block_current: Vec<usize> = block_light
            .iter()
            .enumerate()
            .filter(|&(_, &light)| light > 1)
            .map(|(i, _)| i)
            .collect();

        let mut sky_next: Vec<usize> = Vec::new();
        let mut block_next: Vec<usize> = Vec::new();

        // Process both light types level by level (max 15 levels)
        for _ in 0..15 {
            if sky_current.is_empty() && block_current.is_empty() {
                break;
            }

            // Process sky light level in parallel
            if !sky_current.is_empty() {
                let new_indices: Vec<(usize, u8)> = sky_current
                    .par_iter()
                    .flat_map_iter(|&idx| {
                        let current_light = sky_light[idx];
                        if current_light <= 1 {
                            return Vec::new();
                        }
                        let propagated = current_light - 1;

                        let x = (idx as i32) % world_width;
                        let z = ((idx as i32) / world_width) % world_length;
                        let y = (idx as i32) / (world_width * world_length);

                        let bounds = [
                            x > 0,
                            x < world_width - 1,
                            y > 0,
                            y < total_height - 1,
                            z > 0,
                            z < world_length - 1,
                        ];

                        let mut result = Vec::new();
                        for (i, &offset) in offsets.iter().enumerate() {
                            if bounds[i] {
                                let n_idx = (idx as i32 + offset) as usize;
                                if transparent[n_idx] != 0 && sky_light[n_idx] < propagated {
                                    result.push((n_idx, propagated));
                                }
                            }
                        }
                        result
                    })
                    .collect();

                sky_next.clear();
                for (n_idx, propagated) in new_indices {
                    if sky_light[n_idx] < propagated {
                        sky_light[n_idx] = propagated;
                        sky_next.push(n_idx);
                    }
                }
                std::mem::swap(&mut sky_current, &mut sky_next);
            }

            // Process block light level in parallel
            if !block_current.is_empty() {
                let new_indices: Vec<(usize, u8)> = block_current
                    .par_iter()
                    .flat_map_iter(|&idx| {
                        let current_light = block_light[idx];
                        if current_light <= 1 {
                            return Vec::new();
                        }
                        let propagated = current_light - 1;

                        let x = (idx as i32) % world_width;
                        let z = ((idx as i32) / world_width) % world_length;
                        let y = (idx as i32) / (world_width * world_length);

                        let bounds = [
                            x > 0,
                            x < world_width - 1,
                            y > 0,
                            y < total_height - 1,
                            z > 0,
                            z < world_length - 1,
                        ];

                        let mut result = Vec::new();
                        for (i, &offset) in offsets.iter().enumerate() {
                            if bounds[i] {
                                let n_idx = (idx as i32 + offset) as usize;
                                if transparent[n_idx] != 0 && block_light[n_idx] < propagated {
                                    result.push((n_idx, propagated));
                                }
                            }
                        }
                        result
                    })
                    .collect();

                block_next.clear();
                for (n_idx, propagated) in new_indices {
                    if block_light[n_idx] < propagated {
                        block_light[n_idx] = propagated;
                        block_next.push(n_idx);
                    }
                }
                std::mem::swap(&mut block_current, &mut block_next);
            }
        }
    }

    /// Convert a light volume to per-chunk sections using parallel processing
    fn convert_to_chunk_sections(
        light_volume: &[u8],
        size_in_chunks: Coordinates,
        chunk_column_count: usize,
        world_width: i32,
        world_length: i32,
        total_height: i32,
        default_light: u8,
    ) -> Vec<Vec<LightSection>> {
        (0..chunk_column_count)
            .into_par_iter()
            .map(|chunk_idx| {
                let chunk_x = (chunk_idx as i32) / size_in_chunks.z();
                let chunk_z = (chunk_idx as i32) % size_in_chunks.z();

                let mut sections = Vec::with_capacity(size_in_chunks.y() as usize);

                for section_y in 0..size_in_chunks.y() {
                    let mut light_data = vec![0i8; 2048];
                    let section_base_y = section_y * 16;

                    for local_y in 0..16 {
                        for local_z in 0..16 {
                            for local_x in 0..16 {
                                let world_x = chunk_x * 16 + local_x;
                                let world_y = section_base_y + local_y;
                                let world_z = chunk_z * 16 + local_z;

                                let light_level = if world_x < world_width
                                    && world_z < world_length
                                    && world_y < total_height
                                {
                                    let idx = Self::get_light_index(world_x, world_y, world_z, world_width, world_length);
                                    light_volume[idx]
                                } else {
                                    default_light
                                };

                                let index = ((local_y << 8) | (local_z << 4) | local_x) as usize;
                                let byte_index = index / 2;
                                let is_high_nibble = (index % 2) == 1;

                                if is_high_nibble {
                                    light_data[byte_index] |= (light_level << 4) as i8;
                                } else {
                                    light_data[byte_index] |= light_level as i8;
                                }
                            }
                        }
                    }

                    sections.push(light_data);
                }

                sections
            })
            .collect()
    }

    /// Check if chunk column coordinates (x, z) are within bounds.
    fn is_chunk_column_in_bounds(&self, chunk_x: i32, chunk_z: i32) -> bool {
        chunk_x >= 0
            && chunk_x < self.size_in_chunks.x()
            && chunk_z >= 0
            && chunk_z < self.size_in_chunks.z()
    }

    /// Check if section coordinates (x, y, z) are within bounds.
    fn is_section_in_bounds(&self, chunk_coords: &Coordinates) -> bool {
        chunk_coords.x() >= 0
            && chunk_coords.x() < self.size_in_chunks.x()
            && chunk_coords.y() >= 0
            && chunk_coords.y() < self.size_in_chunks.y()
            && chunk_coords.z() >= 0
            && chunk_coords.z() < self.size_in_chunks.z()
    }

    /// Get the chunk column index for the given (x, z) coordinates.
    fn get_chunk_column_index(&self, chunk_x: i32, chunk_z: i32) -> usize {
        (chunk_z + chunk_x * self.size_in_chunks.z()) as usize
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
