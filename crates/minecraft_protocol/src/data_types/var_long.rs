use crate::prelude::{DecodePacket, EncodePacket};
use pico_binutils::prelude::{
    BinaryReader, BinaryReaderError, BinaryWriter, BinaryWriterError, VarLong,
};
use protocol_version::protocol_version::ProtocolVersion;

impl DecodePacket for VarLong {
    fn decode(
        reader: &mut BinaryReader,
        _protocol_version: ProtocolVersion,
    ) -> Result<Self, BinaryReaderError> {
        reader.read()
    }
}

impl EncodePacket for VarLong {
    fn encode(
        &self,
        writer: &mut BinaryWriter,
        _protocol_version: ProtocolVersion,
    ) -> Result<(), BinaryWriterError> {
        writer.write(self)
    }
}
