use crate::prelude::{Prefixed, VarInt};

/// Strings and Arrays in Network format are prefixed with their length as a VarInt
pub type VarIntPrefixed<T> = Prefixed<VarInt, T>;
