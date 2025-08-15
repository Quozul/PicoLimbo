use crate::prelude::{EncodePacket, LengthPaddedVec};
use pico_binutils::prelude::{BinaryWriter, BinaryWriterError};
use protocol_version::protocol_version::ProtocolVersion;

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
