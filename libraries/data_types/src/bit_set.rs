use crate::encode_packet_field::EncodePacketField;
use crate::length_padded_vec::{LengthPaddedVec, LengthPaddedVecEncodeError};

#[derive(Debug, Default)]
pub struct BitSet {
    data: LengthPaddedVec<i64>,
}

impl EncodePacketField for BitSet {
    type Error = LengthPaddedVecEncodeError;

    fn encode(&self, bytes: &mut Vec<u8>) -> Result<(), Self::Error> {
        self.data.encode(bytes)?;
        Ok(())
    }
}
