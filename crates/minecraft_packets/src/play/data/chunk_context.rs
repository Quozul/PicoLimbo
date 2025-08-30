use crate::play::Coordinates;
use minecraft_protocol::prelude::{Dimension, ProtocolVersion};
use pico_structures::prelude::Schematic;

pub struct VoidChunkContext {
    pub chunk_x: i32,
    pub chunk_z: i32,
    pub biome_index: i32,
    pub dimension: Dimension,
    pub protocol_version: ProtocolVersion,
}

pub struct SchematicChunkContext<'a> {
    pub schematic: &'a Schematic,
    pub paste_origin: Coordinates,
}
