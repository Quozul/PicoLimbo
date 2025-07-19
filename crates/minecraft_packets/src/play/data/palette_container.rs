use minecraft_protocol::prelude::*;
use minecraft_protocol::protocol_version::ProtocolVersion;
use thiserror::Error;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum PaletteContainer {
    SingleValued {
        /// Should always be 0 for Single valued palette
        bits_per_entry: u8,
        value: VarInt,
        /// Present but of length 0 when Bits Per Entry is 0.
        data: Vec<i64>,
    },
    Indirect {
        /// Should be 4-8 for blocks or 1-3 for biomes
        bits_per_entry: u8,
        /// Mapping of IDs in the registry to indices of this array.
        palette: LengthPaddedVec<VarInt>,
        data: Vec<i64>,
    },
    /// Registry IDs are stored directly as entries in the Data Array.
    Direct {
        /// Should be 15 for blocks or 6 for biomes
        bits_per_entry: u8,
        data: Vec<i64>,
    },
}

impl PaletteContainer {
    pub fn blocks_void() -> Self {
        Self::SingleValued {
            bits_per_entry: 0,
            value: VarInt::default(),
            data: Vec::new(),
        }
    }

    pub fn single_valued(value: VarInt) -> Self {
        Self::SingleValued {
            bits_per_entry: 0,
            value,
            data: Vec::new(),
        }
    }
}

#[derive(Error, Debug)]
pub enum PaletteContainerError {
    #[error("error while encoding data field")]
    VecEncodeError(#[from] VecEncodeError),
    #[error("error while encoding palette field")]
    LengthPaddedVecEncodeError(#[from] LengthPaddedVecEncodeError),
    #[error("invalid palette container error")]
    Infallible(#[from] std::convert::Infallible),
}

impl EncodePacketField for PaletteContainer {
    type Error = PaletteContainerError;

    fn encode(&self, bytes: &mut Vec<u8>, protocol_version: i32) -> Result<(), Self::Error> {
        match self {
            PaletteContainer::SingleValued {
                bits_per_entry,
                value,
                data,
            } => {
                bits_per_entry.encode(bytes, protocol_version)?;
                value.encode(bytes, protocol_version)?;
                if protocol_version < ProtocolVersion::V1_21_5.version_number() {
                    VarInt::new(data.len() as i32).encode(bytes, protocol_version)?;
                }
                data.encode(bytes, protocol_version)?;
            }
            PaletteContainer::Indirect {
                bits_per_entry,
                palette,
                data,
            } => {
                bits_per_entry.encode(bytes, protocol_version)?;
                palette.encode(bytes, protocol_version)?;
                if protocol_version < ProtocolVersion::V1_21_5.version_number() {
                    VarInt::new(data.len() as i32).encode(bytes, protocol_version)?;
                }
                data.encode(bytes, protocol_version)?;
            }
            PaletteContainer::Direct {
                bits_per_entry,
                data,
            } => {
                bits_per_entry.encode(bytes, protocol_version)?;
                if protocol_version < ProtocolVersion::V1_21_5.version_number() {
                    VarInt::new(data.len() as i32).encode(bytes, protocol_version)?;
                }
                data.encode(bytes, protocol_version)?;
            }
        }
        Ok(())
    }
}
