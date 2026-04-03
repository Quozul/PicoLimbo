use blocks_report::BlocksReportId;
use minecraft_protocol::prelude::Coordinates;
use pico_structures::prelude::World;
use std::sync::Arc;

pub struct VoidChunkContext {
    pub chunk_x: i32,
    pub chunk_z: i32,
    pub biome_index: i32,
    pub dimension_height: i32,
    pub dimension_min_y: i32,
}

pub struct WorldContext {
    pub world: Arc<World>,
    pub paste_origin: Coordinates,
    pub report_id_mapping: Arc<Vec<BlocksReportId>>,
}
