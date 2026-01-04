use pico_identifier::Identifier;
use thiserror::Error;

#[derive(Default, Copy, Clone)]
#[repr(i8)]
pub enum Dimension {
    #[default]
    Overworld = 0,
    Nether = -1,
    End = 1,
}

impl Dimension {
    #[must_use]
    pub fn identifier(&self) -> Identifier {
        match self {
            Self::Overworld => Identifier::vanilla_unchecked("overworld"),
            Self::Nether => Identifier::vanilla_unchecked("the_nether"),
            Self::End => Identifier::vanilla_unchecked("the_end"),
        }
    }
}

#[derive(Error, Debug)]
#[error("Unknown dimension: {0}")]
pub struct UnknownDimensionError(String);

impl TryFrom<Identifier> for Dimension {
    type Error = UnknownDimensionError;

    fn try_from(identifier: Identifier) -> Result<Self, Self::Error> {
        match identifier.to_string().as_str() {
            "minecraft:overworld" => Ok(Self::Overworld),
            "minecraft:the_nether" => Ok(Self::Nether),
            "minecraft:the_end" => Ok(Self::End),
            _ => Err(UnknownDimensionError(identifier.to_string())),
        }
    }
}
