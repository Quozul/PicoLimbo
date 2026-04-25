use minecraft_protocol::prelude::*;

#[derive(Debug, Clone, Default)]
pub struct Slot {
    item_count: VarInt,
    item_id: Option<VarInt>,
    components_to_add_count: Option<VarInt>,
    components_to_remove_count: Option<VarInt>,
}

impl Slot {
    pub fn empty() -> Self {
        Self {
            item_count: VarInt::new(0),
            item_id: None,
            components_to_add_count: None,
            components_to_remove_count: None,
        }
    }

    pub fn new(item_id: i32, count: i32) -> Self {
        Self {
            item_count: VarInt::new(count),
            item_id: Some(VarInt::new(item_id)),
            components_to_add_count: Some(VarInt::new(0)),
            components_to_remove_count: Some(VarInt::new(0)),
        }
    }
}
