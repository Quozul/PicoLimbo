use minecraft_protocol::prelude::*;
use thiserror::Error;

#[derive(Default, Debug, Clone, Copy)]
pub enum Dimension {
    #[default]
    Overworld,
    Nether,
    End,
}

#[derive(Error, Debug)]
pub enum DimensionError {
    #[error("unknown dimension {0}")]
    UnknownDimension(String),
}

impl Dimension {
    /// Old i8 dimension ID (pre-1.9 client)
    pub fn legacy_i8(self) -> i8 {
        match self {
            Dimension::Overworld => 0,
            Dimension::Nether => -1,
            Dimension::End => 1,
        }
    }

    /// Old i32 dimension ID (1.9+ but not VarInt)
    pub fn legacy_i32(self) -> i32 {
        self.legacy_i8() as i32
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

    pub fn from_name(name: &str) -> Result<Dimension, DimensionError> {
        match name {
            "overworld" => Ok(Dimension::Overworld),
            "nether" => Ok(Dimension::Nether),
            "end" => Ok(Dimension::End),
            _ => Err(DimensionError::UnknownDimension(name.to_string())),
        }
    }
}
