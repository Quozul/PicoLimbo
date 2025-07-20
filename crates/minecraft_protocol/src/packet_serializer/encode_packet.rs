use crate::protocol_version::ProtocolVersion;
use pico_binutils::prelude::{BinaryWriter, BinaryWriterError};

pub trait EncodePacket: Sized {
    fn encode(
        &self,
        writer: &mut BinaryWriter,
        protocol_version: ProtocolVersion,
    ) -> Result<(), BinaryWriterError>;
}
