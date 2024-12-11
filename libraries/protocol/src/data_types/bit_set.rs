use crate::prelude::{LengthPaddedVec, LengthPaddedVecEncodeError, SerializePacketData};

#[derive(Debug, Default)]
pub struct BitSet {
    data: LengthPaddedVec<i64>,
}

impl SerializePacketData for BitSet {
    type Error = LengthPaddedVecEncodeError;

    fn encode(&self, bytes: &mut Vec<u8>) -> Result<(), Self::Error> {
        self.data.encode(bytes)?;
        Ok(())
    }
}
