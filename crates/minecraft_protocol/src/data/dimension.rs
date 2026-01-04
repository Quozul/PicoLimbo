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
    pub const ALL_DIMENSIONS: &'static [Self] = &[Self::Overworld, Self::Nether, Self::End];

    pub const fn legacy_i8(self) -> i8 {
        self as i8
    }

    /// 1.20.5 dimension_type registry index
    ///   0: overworld, 1: overworld_caves, 2: the_end, 3: the_nether
    pub fn type_index_1_20_5(self) -> VarInt {
        let idx = match self {
            Self::Overworld => 0,
            Self::Nether => 3,
            Self::End => 2,
        };
        VarInt::new(idx)
    }

    /// Always use the vanilla identifier for name and dimension_type in 1.16+ clients
    pub fn identifier(self) -> Identifier {
        match self {
            Self::Overworld => Identifier::vanilla_unchecked("overworld"),
            Self::Nether => Identifier::vanilla_unchecked("the_nether"),
            Self::End => Identifier::vanilla_unchecked("the_end"),
        }
    }
}

#[derive(Debug, Error)]
#[error("Dimension {0} is invalid")]
pub struct InvalidDimension(String);

impl FromStr for Dimension {
    type Err = InvalidDimension;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "minecraft:overworld" => Ok(Self::Overworld),
            "minecraft:the_nether" => Ok(Self::Nether),
            "minecraft:the_end" => Ok(Self::End),
            _ => Err(InvalidDimension(s.to_string())),
        }
    }
}

impl Display for Dimension {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Overworld => write!(f, "Overworld"),
            Self::Nether => write!(f, "Nether"),
            Self::End => write!(f, "End"),
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
