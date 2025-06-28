use crate::prelude::{EncodePacketField, LengthPaddedVec, LengthPaddedVecEncodeError};

#[derive(Debug, Clone, Default)]
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

impl EncodePacketField for BitSet {
    type Error = LengthPaddedVecEncodeError;

    fn encode(&self, bytes: &mut Vec<u8>, protocol_version: u32) -> Result<(), Self::Error> {
        self.data.encode(bytes, protocol_version)?;
        Ok(())
    }
}
