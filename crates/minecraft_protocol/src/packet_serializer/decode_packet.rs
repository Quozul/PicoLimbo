use pico_binutils::prelude::{BinaryReader, BinaryReaderError};
use protocol_version::protocol_version::ProtocolVersion;

pub trait DecodePacket: Sized {
    fn decode(
        reader: &mut BinaryReader,
        protocol_version: ProtocolVersion,
    ) -> Result<Self, BinaryReaderError>;
}
