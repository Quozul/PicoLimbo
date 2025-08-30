use blocks_report::InternalId;

pub enum Palette {
    Single {
        internal_id: InternalId, // Must be remapped before sending
    },
    Paletted {
        bits_per_entry: u8,
        internal_palette: Vec<InternalId>, // Only the internal palette must be remapped before sending
        packed_data: Vec<u64>,
    },
    Direct {
        internal_data: Vec<InternalId>, // Data must be remapped and packet before sending
    },
}

impl Palette {
    pub fn single(internal_id: InternalId) -> Self {
        Self::Single { internal_id }
    }

    pub fn paletted(
        bits_per_entry: u8,
        internal_palette: Vec<InternalId>,
        packed_data: Vec<u64>,
    ) -> Self {
        Self::Paletted {
            bits_per_entry,
            internal_palette,
            packed_data,
        }
    }

    pub fn direct(internal_data: Vec<InternalId>) -> Self {
        Self::Direct { internal_data }
    }
}
