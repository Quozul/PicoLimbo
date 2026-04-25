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

    pub fn get_block_at(&self, local_x: i32, local_y: i32, local_z: i32) -> Option<InternalId> {
        let index = (local_y * 256 + local_z * 16 + local_x) as usize;
        self.get_block_at_index(index)
    }

    pub fn get_block_at_index(&self, index: usize) -> Option<InternalId> {
        if index >= 4096 {
            return None;
        }

        match self {
            Palette::Single { internal_id } => Some(*internal_id),
            Palette::Direct { internal_data } => internal_data.get(index).copied(),
            Palette::Paletted {
                bits_per_entry,
                internal_palette,
                packed_data,
            } => {
                let bits = *bits_per_entry as usize;
                let entries_per_long = 64 / bits;
                let long_index = index / entries_per_long;
                let offset = (index % entries_per_long) * bits;
                let mask = (1u64 << bits) - 1;

                packed_data.get(long_index).and_then(|&long_value| {
                    let palette_index = ((long_value >> offset) & mask) as usize;
                    internal_palette.get(palette_index).copied()
                })
            }
        }
    }
}
