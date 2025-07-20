use crate::protocol_version::ProtocolVersion;
use pico_binutils::prelude::{BinaryReader, BinaryReaderError};

pub trait DecodePacket: Sized {
    fn decode(
        reader: &mut BinaryReader,
        protocol_version: ProtocolVersion,
    ) -> Result<Self, BinaryReaderError>;
}
