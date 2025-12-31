use std::fmt::{Display, Formatter};
use std::marker::PhantomData;

#[derive(Debug, PartialEq, Eq, Clone, Default, Hash)]
pub struct Prefixed<L, T>(pub(crate) T, pub(crate) PhantomData<L>);

impl<L> Prefixed<L, String> {
    pub fn string(str: impl ToString) -> Self {
        Self(str.to_string(), PhantomData)
    }
}

impl<L> Display for Prefixed<L, String> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0.as_str())
    }
}

impl<L, T> Prefixed<L, T> {
    pub fn new(value: T) -> Self {
        Self(value, PhantomData)
    }

    pub fn into_inner(self) -> T {
        self.0
    }

    pub fn inner(&self) -> &T {
        &self.0
    }

    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

/// Arrays in NBT format are prefixed with their length as an Integer
pub type IntPrefixed<T> = Prefixed<i32, T>;

/// Strings in NBT format are prefixed with their length as a UShort
pub type UShortPrefixed<T> = Prefixed<u16, T>;
