/// Options for NBT decoding and encoding.
#[derive(Debug, Clone, Copy, Default)]
pub struct NbtOptions {
    flags: u8,
}

const NAMELESS_ROOT: u8 = 1 << 0;
const DYNAMIC_LISTS: u8 = 1 << 1;

impl NbtOptions {
    /// Creates a new `NbtOptions` with default settings.
    #[must_use]
    pub const fn new() -> Self {
        Self { flags: 0 }
    }

    /// Sets whether the root tag has a name.
    ///
    /// Since Minecraft 1.20.2, NBT sent over the network does not have a name for the root tag.
    /// If this is true, the root tag name is skipped during decoding and encoding.
    #[must_use]
    pub const fn nameless_root(mut self, enabled: bool) -> Self {
        if enabled {
            self.flags |= NAMELESS_ROOT;
        } else {
            self.flags &= !NAMELESS_ROOT;
        }
        self
    }

    /// Sets whether to support heterogeneous lists (dynamic lists).
    ///
    /// Since Minecraft 1.21.5, lists can contain elements of different types.
    /// If this is true, heterogeneous lists are encoded as a list of compounds,
    /// where each compound has a single empty key containing the value.
    #[must_use]
    pub const fn dynamic_lists(mut self, enabled: bool) -> Self {
        if enabled {
            self.flags |= DYNAMIC_LISTS;
        } else {
            self.flags &= !DYNAMIC_LISTS;
        }
        self
    }

    /// Checks if nameless root is enabled.
    #[must_use]
    pub const fn is_nameless_root(&self) -> bool {
        (self.flags & NAMELESS_ROOT) != 0
    }

    /// Checks if dynamic lists are enabled.
    #[must_use]
    pub const fn is_dynamic_lists(&self) -> bool {
        (self.flags & DYNAMIC_LISTS) != 0
    }
}
