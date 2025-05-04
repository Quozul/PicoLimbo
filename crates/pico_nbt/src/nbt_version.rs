#[derive(Default, Copy, Clone, Debug, PartialEq, Eq)]
pub struct NbtFeatures(u8);

impl NbtFeatures {
    const NAMELESS_BIT: u8 = 1 << 1;
    const DYNAMIC_LISTS_BIT: u8 = 1 << 2;

    pub const fn all() -> Self {
        Self(Self::NAMELESS_BIT | Self::DYNAMIC_LISTS_BIT)
    }

    pub fn builder() -> NbtFeaturesBuilder {
        NbtFeaturesBuilder::default()
    }

    pub fn is_nameless_available(&self) -> bool {
        (self.0 & Self::NAMELESS_BIT) != 0
    }

    pub fn is_dynamic_lists_available(&self) -> bool {
        (self.0 & Self::DYNAMIC_LISTS_BIT) != 0
    }
}

#[derive(Default)]
pub struct NbtFeaturesBuilder(u8);

impl NbtFeaturesBuilder {
    /// Since 1.20.2, NBT sent over the network has been updated to exclude the name from the root tag.
    pub fn nameless(&mut self) -> &mut Self {
        self.0 |= NbtFeatures::NAMELESS_BIT;
        self
    }

    /// As of 1.21.5, all NBT components now supports heterogenous (differently-typed) lists.
    pub fn dynamic_lists(&mut self) -> &mut Self {
        self.0 |= NbtFeatures::DYNAMIC_LISTS_BIT;
        self
    }

    pub fn build(&self) -> NbtFeatures {
        NbtFeatures(self.0)
    }
}
