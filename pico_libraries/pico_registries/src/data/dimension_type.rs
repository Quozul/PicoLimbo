use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct DimensionType {
    height: i32,
    min_y: i32,
}

impl DimensionType {
    pub const fn get_height(&self) -> i32 {
        self.height
    }

    pub const fn get_min_height(&self) -> i32 {
        self.min_y
    }
}
