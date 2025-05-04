use crate::prelude::NbtFeatures;

pub struct NbtContext {
    is_root: bool,
    is_in_list: bool,
}

impl Default for NbtContext {
    fn default() -> Self {
        Self {
            is_root: false,
            is_in_list: false,
        }
    }
}

impl NbtContext {
    pub fn root() -> Self {
        Self {
            is_root: true,
            is_in_list: false,
        }
    }

    pub fn list() -> Self {
        Self {
            is_root: false,
            is_in_list: true,
        }
    }

    pub fn should_skip_name(&self, nbt_features: NbtFeatures) -> bool {
        nbt_features.is_nameless_available() && self.is_root || self.is_in_list
    }

    pub fn should_skip_tag_type(&self) -> bool {
        self.is_in_list
    }
}
