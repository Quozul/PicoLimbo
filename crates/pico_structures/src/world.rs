use crate::chunk_processor::{ChunkProcessor, ChunkProcessorError};
use crate::internal_block_entity::BlockEntity;
use crate::palette::Palette;
use crate::prelude::Schematic;
use blocks_report::InternalId;
use minecraft_protocol::prelude::Coordinates;
use rayon::iter::ParallelIterator;
use rayon::prelude::IntoParallelIterator;
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
        let block_light_atomic: Vec<AtomicU8> =
            (0..volume_size).map(|_| AtomicU8::new(0)).collect();

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

        let mut sky_light_volume: Vec<u8> = sky_light_atomic
            .into_iter()
            .map(|a| a.into_inner())
            .collect();
        let mut block_light_volume: Vec<u8> = block_light_atomic
            .into_iter()
            .map(|a| a.into_inner())
            .collect();

        // Propagate light using Starlight-style algorithm (propagate TO neighbors, track direction)
        Self::propagate_light_starlight(
            &mut sky_light_volume,
            &transparent,
            world_width,
            world_length,
            total_height,
        );
        Self::propagate_light_starlight(
            &mut block_light_volume,
            &transparent,
            world_width,
            world_length,
            total_height,
        );

        let sky_light_by_chunk = Self::convert_to_chunk_sections(
            &sky_light_volume,
            size_in_chunks,
            chunk_column_count,
            world_width,
            world_length,
            total_height,
            15,
        );
        let block_light_by_chunk = Self::convert_to_chunk_sections(
            &block_light_volume,
            size_in_chunks,
            chunk_column_count,
            world_width,
            world_length,
            total_height,
            0,
        );

        (sky_light_by_chunk, block_light_by_chunk)
    }

    #[inline]
    fn get_light_index(x: i32, y: i32, z: i32, world_width: i32, world_length: i32) -> usize {
        (y * world_width * world_length + z * world_width + x) as usize
    }

    /// Starlight-style light propagation: propagate TO neighbors, track direction to avoid redundant checks
    /// Queue entry format (packed u64):
    /// - bits 0-31: index in light volume
    /// - bits 32-35: light level to propagate (4 bits, 0-15)
    /// - bits 36-41: direction bitset (6 bits, one per direction to check)
    fn propagate_light_starlight(
        light_volume: &mut [u8],
        transparent: &[u8],
        world_width: i32,
        world_length: i32,
        total_height: i32,
    ) {
        // Direction offsets: -x, +x, -y, +y, -z, +z
        let stride_x = 1i32;
        let stride_z = world_width;
        let stride_y = world_width * world_length;
        let offsets: [i32; 6] = [
            -stride_x, stride_x, -stride_y, stride_y, -stride_z, stride_z,
        ];

        // For each direction, the bitset of directions to check (excludes opposite direction)
        // Direction 0 (-x) opposite is 1 (+x), so exclude bit 1
        // Direction 1 (+x) opposite is 0 (-x), so exclude bit 0
        // etc.
        const ALL_DIRS: u8 = 0b111111;
        let exclude_opposite: [u8; 6] = [
            ALL_DIRS ^ (1 << 1), // came from -x, don't check +x
            ALL_DIRS ^ (1 << 0), // came from +x, don't check -x
            ALL_DIRS ^ (1 << 3), // came from -y, don't check +y
            ALL_DIRS ^ (1 << 2), // came from +y, don't check -y
            ALL_DIRS ^ (1 << 5), // came from -z, don't check +z
            ALL_DIRS ^ (1 << 4), // came from +z, don't check -z
        ];

        // Initialize queue with all light sources (light > 1, since level 1 can't propagate)
        let mut queue: Vec<u64> = light_volume
            .iter()
            .enumerate()
            .filter(|&(_, &light)| light > 1)
            .map(|(idx, &light)| {
                // Initial sources check all 6 directions
                (idx as u64) | ((light as u64) << 32) | ((ALL_DIRS as u64) << 36)
            })
            .collect();

        let mut read_idx = 0;

        while read_idx < queue.len() {
            let entry = queue[read_idx];
            read_idx += 1;

            let idx = (entry & 0xFFFFFFFF) as usize;
            let propagated_level = ((entry >> 32) & 0xF) as u8;
            let check_dirs = ((entry >> 36) & 0x3F) as u8;

            if propagated_level <= 1 {
                continue;
            }

            let target_level = propagated_level - 1;

            // Compute position for bounds checking
            let x = (idx as i32) % world_width;
            let z = ((idx as i32) / world_width) % world_length;
            let y = (idx as i32) / (world_width * world_length);

            // Check each direction in the bitset
            for dir in 0..6u8 {
                if (check_dirs & (1 << dir)) == 0 {
                    continue;
                }

                // Bounds check
                let in_bounds = match dir {
                    0 => x > 0,                // -x
                    1 => x < world_width - 1,  // +x
                    2 => y > 0,                // -y
                    3 => y < total_height - 1, // +y
                    4 => z > 0,                // -z
                    _ => z < world_length - 1, // +z
                };

                if !in_bounds {
                    continue;
                }

                let n_idx = (idx as i32 + offsets[dir as usize]) as usize;

                // Only propagate to transparent blocks
                if transparent[n_idx] == 0 {
                    continue;
                }

                // Only update if we would increase the light level
                let current_level = light_volume[n_idx];
                if current_level >= target_level {
                    continue;
                }

                // Set the new light level
                light_volume[n_idx] = target_level;

                // Queue this neighbor to propagate further (if it can)
                if target_level > 1 {
                    let next_dirs = exclude_opposite[dir as usize];
                    queue.push(
                        (n_idx as u64) | ((target_level as u64) << 32) | ((next_dirs as u64) << 36),
                    );
                }
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
                                    let idx = Self::get_light_index(
                                        world_x,
                                        world_y,
                                        world_z,
                                        world_width,
                                        world_length,
                                    );
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

    /// Get the InternalId of the block at the given world position.
    pub fn get_block_at(&self, x: i32, y: i32, z: i32) -> Option<InternalId> {
        let section_coords = Coordinates::new(x.div_euclid(16), y.div_euclid(16), z.div_euclid(16));
        self.get_section(&section_coords).and_then(|palette| {
            palette.get_block_at(x.rem_euclid(16), y.rem_euclid(16), z.rem_euclid(16))
        })
    }

    /// Get the BlockEntity at the given world position, if one exists.
    pub fn get_block_entity_at(&self, x: i32, y: i32, z: i32) -> Option<&BlockEntity> {
        let chunk_x = x.div_euclid(16);
        let chunk_z = z.div_euclid(16);

        self.get_chunk_block_entities(chunk_x, chunk_z)
            .and_then(|entities| {
                entities.iter().find(|e| {
                    let pos = e.get_position();
                    pos.x() == x && pos.y() == y && pos.z() == z
                })
            })
    }
}
