use pico_binutils::prelude::{BinaryWriter, BinaryWriterError};
use protocol_version::protocol_version::ProtocolVersion;

pub trait EncodePacket: Sized {
    fn encode(
        &self,
        writer: &mut BinaryWriter,
        protocol_version: ProtocolVersion,
    ) -> Result<(), BinaryWriterError>;
}
