use crate::prelude::*;
use std::fmt::Display;
use std::str::FromStr;
use thiserror::Error;

#[derive(Default, Clone, Copy)]
#[repr(i8)]
pub enum Dimension {
    #[default]
    Overworld = 0,
    Nether = -1,
    End = 1,
}

impl Dimension {
    pub const ALL_DIMENSIONS: &'static [Dimension] =
        &[Dimension::Overworld, Dimension::Nether, Dimension::End];

    pub const fn legacy_i8(self) -> i8 {
        self as i8
    }

    /// 1.20.5 dimension_type registry index
    ///   0: overworld, 1: overworld_caves, 2: the_end, 3: the_nether
    pub fn type_index_1_20_5(self) -> VarInt {
        let idx = match self {
            Dimension::Overworld => 0,
            Dimension::Nether => 3,
            Dimension::End => 2,
        };
        VarInt::new(idx)
    }

    /// Always use the vanilla identifier for name and dimension_type in 1.16+ clients
    pub fn identifier(self) -> Identifier {
        match self {
            Dimension::Overworld => Identifier::minecraft("overworld"),
            Dimension::Nether => Identifier::minecraft("the_nether"),
            Dimension::End => Identifier::minecraft("the_end"),
        }
    }

    #[inline]
    pub const fn height(self) -> i32 {
        256
    }

    #[inline]
    pub const fn min_y(self) -> i32 {
        0
    }
}

#[derive(Debug, Error)]
#[error("Dimension {0} is invalid")]
pub struct InvalidDimension(String);

impl FromStr for Dimension {
    type Err = InvalidDimension;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "minecraft:overworld" => Ok(Dimension::Overworld),
            "minecraft:the_nether" => Ok(Dimension::Nether),
            "minecraft:the_end" => Ok(Dimension::End),
            _ => Err(InvalidDimension(s.to_string())),
        }
    }
}

impl Display for Dimension {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Dimension::Overworld => write!(f, "Overworld"),
            Dimension::Nether => write!(f, "Nether"),
            Dimension::End => write!(f, "End"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_legacy_i8() {
        assert_eq!(Dimension::Overworld.legacy_i8(), 0);
        assert_eq!(Dimension::Nether.legacy_i8(), -1);
        assert_eq!(Dimension::End.legacy_i8(), 1);
    }

    #[test]
    fn test_modern_var_int() {
        assert_eq!(Dimension::Overworld.type_index_1_20_5().inner(), 0);
        assert_eq!(Dimension::Nether.type_index_1_20_5().inner(), 3);
        assert_eq!(Dimension::End.type_index_1_20_5().inner(), 2);
    }
}
