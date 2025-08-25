use std::collections::HashMap;

/// Helper to manage the creation of a de-duplicated palette.
pub struct PaletteTranslator {
    /// The final, ordered list of global block state IDs.
    pub palette_ids: Vec<i32>,
    /// A map for fast, de-duplicated lookups of global IDs to their local index.
    pub global_id_to_local_index: HashMap<i32, i32>,
}

impl PaletteTranslator {
    pub fn new(air_id: i32) -> Self {
        let mut palette_ids = Vec::new();
        let mut global_id_to_local_index = HashMap::new();
        // Ensure air is always at index 0.
        palette_ids.push(air_id);
        global_id_to_local_index.insert(air_id, 0);
        Self {
            palette_ids,
            global_id_to_local_index,
        }
    }

    /// Gets the local index for a global ID, inserting it into the palette if it's new.
    pub fn get_or_insert(&mut self, global_id: i32) -> i32 {
        *self
            .global_id_to_local_index
            .entry(global_id)
            .or_insert_with(|| {
                let next_index = self.palette_ids.len() as i32;
                self.palette_ids.push(global_id);
                next_index
            })
    }
}
