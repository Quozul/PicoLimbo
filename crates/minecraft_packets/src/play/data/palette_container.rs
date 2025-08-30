use minecraft_protocol::prelude::*;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum PaletteContainer {
    SingleValued {
        /// Should always be 0 for Single valued palette
        bits_per_entry: u8,
        value: VarInt,
    },
    Indirect {
        /// Should be 4-8 for blocks or 1-3 for biomes
        bits_per_entry: u8,
        /// Mapping of IDs in the registry to indices of this array.
        palette: LengthPaddedVec<VarInt>,
        data: Vec<u64>,
    },
    /// Registry IDs are stored directly as entries in the Data Array.
    Direct {
        /// Should be 15 for blocks or 6 for biomes
        bits_per_entry: u8,
        data: Vec<u64>,
    },
}

impl PaletteContainer {
    pub fn blocks_void() -> Self {
        Self::single_valued(0)
    }

    pub fn single_valued(value: impl Into<VarInt>) -> Self {
        Self::SingleValued {
            bits_per_entry: 0,
            value: value.into(),
        }
    }
}

impl EncodePacket for PaletteContainer {
    fn encode(
        &self,
        writer: &mut BinaryWriter,
        protocol_version: ProtocolVersion,
    ) -> Result<(), BinaryWriterError> {
        match self {
            PaletteContainer::SingleValued {
                bits_per_entry,
                value,
            } => {
                bits_per_entry.encode(writer, protocol_version)?;
                value.encode(writer, protocol_version)?;
                if protocol_version.is_before_inclusive(ProtocolVersion::V1_21_4) {
                    VarInt::new(0).encode(writer, protocol_version)?;
                }
            }
            PaletteContainer::Indirect {
                bits_per_entry,
                palette,
                data,
            } => {
                bits_per_entry.encode(writer, protocol_version)?;
                palette.encode(writer, protocol_version)?;
                if protocol_version.is_before_inclusive(ProtocolVersion::V1_21_4) {
                    VarInt::new(data.len() as i32).encode(writer, protocol_version)?;
                }
                data.encode(writer, protocol_version)?;
            }
            PaletteContainer::Direct {
                bits_per_entry,
                data,
            } => {
                bits_per_entry.encode(writer, protocol_version)?;
                if protocol_version.is_before_inclusive(ProtocolVersion::V1_21_4) {
                    VarInt::new(data.len() as i32).encode(writer, protocol_version)?;
                }
                for &long_value in data {
                    long_value.encode(writer, protocol_version)?;
                }
            }
        }
        Ok(())
    }
}
