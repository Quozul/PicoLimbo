use crate::server::packet_registry::PacketRegistry;
use blocks_report::get_block_report_id_mapping;
use minecraft_packets::play::chunk_data_and_update_light_packet::ChunkDataAndUpdateLightPacket;
use minecraft_packets::play::{VoidChunkContext, WorldContext};
use minecraft_protocol::prelude::{Coordinates, Dimension, ProtocolVersion};
use pico_structures::prelude::World;
use std::sync::Arc;

#[derive(Copy, Clone)]
enum Direction {
    Right,
    Up,
    Left,
    Down,
}

impl Direction {
    const fn turn(self) -> Self {
        match self {
            Self::Right => Self::Up,
            Self::Up => Self::Left,
            Self::Left => Self::Down,
            Self::Down => Self::Right,
        }
    }

    const fn step(self, x: &mut i32, y: &mut i32) {
        match self {
            Self::Right => *x += 1,
            Self::Up => *y -= 1,
            Self::Left => *x -= 1,
            Self::Down => *y += 1,
        }
    }
}

struct SpiralIterator {
    center_x: i32,
    center_y: i32,
    current_x: i32,
    current_y: i32,
    direction: Direction,
    leg_length: i32,
    steps_remaining_in_leg: i32,
    grow_next_leg: bool,
    max_radius: i32,
}

impl SpiralIterator {
    const fn new(center_x: i32, center_y: i32, max_radius: i32) -> Self {
        Self {
            center_x,
            center_y,
            current_x: center_x,
            current_y: center_y,
            direction: Direction::Right,
            leg_length: 1,
            steps_remaining_in_leg: 1,
            grow_next_leg: false,
            max_radius,
        }
    }
}

impl Iterator for SpiralIterator {
    type Item = (i32, i32);

    fn next(&mut self) -> Option<Self::Item> {
        // Stop when the next position to yield is outside the allowed radius.
        let distance_x = (self.current_x - self.center_x).abs();
        let distance_y = (self.current_y - self.center_y).abs();
        if distance_x.max(distance_y) > self.max_radius {
            return None;
        }

        // Yield current position.
        let result = (self.current_x, self.current_y);

        // Advance state for the next call.
        self.direction
            .step(&mut self.current_x, &mut self.current_y);
        self.steps_remaining_in_leg -= 1;

        if self.steps_remaining_in_leg == 0 {
            self.direction = self.direction.turn();

            if self.grow_next_leg {
                self.leg_length += 1;
            }
            self.grow_next_leg = !self.grow_next_leg;

            self.steps_remaining_in_leg = self.leg_length;
        }

        Some(result)
    }
}

pub struct CircularChunkPacketIterator {
    biome_index: i32,
    dimension: Dimension,
    schematic_context: Option<WorldContext>,
    spiral_iterator: SpiralIterator,
    protocol_version: ProtocolVersion,
}

impl CircularChunkPacketIterator {
    pub fn new(
        center_chunk: (i32, i32),
        view_distance: i32,
        world: Option<Arc<World>>,
        biome_index: i32,
        dimension: Dimension,
        protocol_version: ProtocolVersion,
    ) -> Self {
        let (center_x, center_z) = center_chunk;
        let paste_origin = Coordinates::new_uniform(0);

        let schematic_context: Option<WorldContext> = get_block_report_id_mapping(protocol_version)
            .map_or(None, |report_id_mapping| {
                world.map(|world_arc| WorldContext {
                    paste_origin,
                    world: world_arc,
                    report_id_mapping: Arc::new(report_id_mapping),
                })
            });

        Self {
            biome_index,
            dimension,
            schematic_context,
            spiral_iterator: SpiralIterator::new(center_x, center_z, view_distance),
            protocol_version,
        }
    }
}

impl Iterator for CircularChunkPacketIterator {
    type Item = PacketRegistry;

    fn next(&mut self) -> Option<Self::Item> {
        let (chunk_x, chunk_z) = self.spiral_iterator.next()?;

        let chunk_context = VoidChunkContext {
            chunk_x,
            chunk_z,
            biome_index: self.biome_index,
            dimension: self.dimension,
        };

        let packet = match &self.schematic_context {
            Some(context) => ChunkDataAndUpdateLightPacket::from_structure(
                chunk_context,
                context,
                self.protocol_version,
            ),
            None => ChunkDataAndUpdateLightPacket::void(chunk_context),
        };

        Some(PacketRegistry::ChunkDataAndUpdateLight(Box::new(packet)))
    }
}
