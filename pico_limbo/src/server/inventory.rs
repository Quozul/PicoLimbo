#[derive(Debug, Clone)]
pub struct Inventory {
    /// The currently selected hotbar slot (0-8).
    current_slot: i16,
}

impl Default for Inventory {
    fn default() -> Self {
        Self { current_slot: 0 }
    }
}

impl Inventory {
    pub const fn current_slot(&self) -> i16 {
        self.current_slot
    }

    pub const fn set_current_slot(&mut self, slot: i16) {
        self.current_slot = slot;
    }
}
