use crate::prelude::{EncodePacket, LengthPaddedVec};
use crate::protocol_version::ProtocolVersion;
use pico_binutils::prelude::{BinaryWriter, BinaryWriterError};

#[derive(Debug, Default)]
pub struct BitSet {
    data: LengthPaddedVec<i64>,
}

impl EncodePacket for BitSet {
    fn encode(
        &self,
        writer: &mut BinaryWriter,
        _protocol_version: ProtocolVersion,
    ) -> Result<(), BinaryWriterError> {
        writer.write(&self.data)
    }
}
