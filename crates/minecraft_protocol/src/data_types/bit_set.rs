use crate::prelude::*;

#[derive(Default, Clone, PacketOut, PacketIn)]
pub struct BitSet {
    data: LengthPaddedVec<i64>,
}

impl BitSet {
    pub fn new(data: Vec<i64>) -> Self {
        Self {
            data: LengthPaddedVec::new(data),
        }
    }
}
