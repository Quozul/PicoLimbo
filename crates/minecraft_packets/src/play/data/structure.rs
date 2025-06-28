pub struct Structure {
    stone_id: i32,
    air_id: i32,
    plains_biome_id: i32,
}

impl Structure {
    pub fn new(stone_id: i32, air_id: i32, biome_id: i32) -> Self {
        Self {
            stone_id,
            air_id,
            plains_biome_id: biome_id,
        }
    }

    /// Get the block registry ID at the given coordinates within the structure
    /// Coordinates should be in the range 0-15 for a single chunk section
    pub fn get_block_at(&self, x: i32, y: i32, z: i32) -> i32 {
        // stone floor at y=0, stone walls around perimeter
        if y == 0 || y <= 3 && (x == 0 || x == 15 || z == 0 || z == 15) {
            self.stone_id
        } else {
            self.air_id
        }
    }

    /// Get the biome registry ID at the given coordinates within the structure
    /// Coordinates should be in the range 0-15 for a single chunk section
    pub fn get_biome_at(&self, x: i32, y: i32, z: i32) -> i32 {
        self.plains_biome_id
    }
}
