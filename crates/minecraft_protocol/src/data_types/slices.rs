use crate::prelude::EncodePacket;
use pico_binutils::prelude::{BinaryWriter, BinaryWriterError};
use protocol_version::protocol_version::ProtocolVersion;

impl EncodePacket for &'static [u8] {
    fn encode(
        &self,
        writer: &mut BinaryWriter,
        _protocol_version: ProtocolVersion,
    ) -> Result<(), BinaryWriterError> {
        writer.write_bytes(self)?;
        Ok(())
    }
}
