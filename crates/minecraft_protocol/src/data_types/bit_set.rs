use crate::prelude::{EncodePacket, LengthPaddedVec};
use macros::PacketOut;
use pico_binutils::prelude::{BinaryWriter, BinaryWriterError};
use protocol_version::protocol_version::ProtocolVersion;

#[derive(Default, Clone, PacketOut)]
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
