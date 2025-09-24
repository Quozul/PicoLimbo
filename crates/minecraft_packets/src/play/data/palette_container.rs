use blocks_report::{BlocksReportId, InternalId, get_block_id};
use minecraft_protocol::prelude::*;
use pico_structures::prelude::{Palette, pack_direct};

#[derive(Clone)]
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
        data: LengthPaddedVec<u64>,
    },
    /// Registry IDs are stored directly as entries in the Data Array.
    Direct {
        /// Should be 15 for blocks or 6 for biomes
        bits_per_entry: u8,
        data: LengthPaddedVec<u64>,
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

    pub fn from_palette(palette: &Palette, report_id_mapping: &[BlocksReportId]) -> Self {
        const AIR_ID: BlocksReportId = 0;

        let map_id = |internal_id: &InternalId| -> i32 {
            get_block_id(report_id_mapping, *internal_id).unwrap_or(AIR_ID) as i32
        };

        match palette {
            Palette::Single { internal_id } => Self::SingleValued {
                bits_per_entry: 0,
                value: VarInt::new(map_id(internal_id)),
            },
            Palette::Paletted {
                bits_per_entry,
                internal_palette,
                packed_data,
            } => {
                let global_palette = internal_palette
                    .iter()
                    .map(|id| VarInt::new(map_id(id)))
                    .collect();

                Self::Indirect {
                    bits_per_entry: *bits_per_entry,
                    palette: LengthPaddedVec::new(global_palette),
                    data: LengthPaddedVec::new(packed_data.clone()),
                }
            }
            Palette::Direct { internal_data } => {
                const BITS_PER_ENTRY: u8 = 15;

                let global_data_iter = internal_data.iter().map(|id| map_id(id) as u32);

                Self::Direct {
                    bits_per_entry: BITS_PER_ENTRY,
                    data: LengthPaddedVec::new(pack_direct(global_data_iter, BITS_PER_ENTRY)),
                }
            }
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
                if protocol_version.is_after_inclusive(ProtocolVersion::V1_21_5) {
                    data.inner().encode(writer, protocol_version)?;
                } else {
                    data.encode(writer, protocol_version)?;
                }
            }
            PaletteContainer::Direct {
                bits_per_entry,
                data,
            } => {
                bits_per_entry.encode(writer, protocol_version)?;
                if protocol_version.is_after_inclusive(ProtocolVersion::V1_21_5) {
                    data.inner().encode(writer, protocol_version)?;
                } else {
                    data.encode(writer, protocol_version)?;
                }
            }
        }
        Ok(())
    }
}
