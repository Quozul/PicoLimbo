/// Options for NBT decoding and encoding.
#[derive(Debug, Clone, Copy, Default)]
pub struct NbtOptions {
    flags: u8,
}

const NAMELESS_ROOT: u8 = 1 << 0;
const DYNAMIC_LISTS: u8 = 1 << 1;
const NUMERIC_WIDENING: u8 = 1 << 2;

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

    /// Sets whether integer JSON numbers should be widened to canonical NBT types
    /// (`Int` for values fitting i32, `Long` otherwise) and floating-point numbers
    /// should always be encoded as `Float` (32-bit), instead of being downcast to
    /// the smallest-fitting NBT type. Booleans (`JsonValue::Bool`) are unaffected
    /// and remain `Value::Byte(0/1)`.
    ///
    /// This flag matches the NBT serialization Mojang's vanilla server emits for
    /// registry data, and is required to satisfy strict codecs such as
    /// `PacketEvents`' `minecraft:timeline` decoder.
    #[must_use]
    pub const fn numeric_widening(mut self, enabled: bool) -> Self {
        if enabled {
            self.flags |= NUMERIC_WIDENING;
        } else {
            self.flags &= !NUMERIC_WIDENING;
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

    /// Checks if numeric widening is enabled.
    #[must_use]
    pub const fn is_numeric_widening(&self) -> bool {
        (self.flags & NUMERIC_WIDENING) != 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numeric_widening_default_is_false() {
        assert!(!NbtOptions::new().is_numeric_widening());
    }

    #[test]
    fn numeric_widening_can_be_enabled() {
        let opts = NbtOptions::new().numeric_widening(true);
        assert!(opts.is_numeric_widening());
    }

    #[test]
    fn numeric_widening_can_be_disabled() {
        let opts = NbtOptions::new().numeric_widening(true).numeric_widening(false);
        assert!(!opts.is_numeric_widening());
    }

    #[test]
    fn numeric_widening_does_not_collide_with_other_flags() {
        let opts = NbtOptions::new()
            .nameless_root(true)
            .dynamic_lists(true)
            .numeric_widening(true);
        assert!(opts.is_nameless_root());
        assert!(opts.is_dynamic_lists());
        assert!(opts.is_numeric_widening());
    }
}
