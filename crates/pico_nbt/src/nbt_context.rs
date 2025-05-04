use crate::prelude::NbtFeatures;

#[derive(Default)]
pub struct NbtContext {
    is_root: bool,
    is_in_list: bool,
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

    pub fn should_include_tag_name(&self, nbt_features: NbtFeatures) -> bool {
        !(self.is_in_list || nbt_features.is_nameless_available() && self.is_root)
    }

    pub fn should_include_tag_type(&self) -> bool {
        !self.is_in_list
    }
}
