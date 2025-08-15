use crate::prelude::{DecodePacket, EncodePacket};
use pico_binutils::prelude::{
    BinaryReader, BinaryReaderError, BinaryWriter, BinaryWriterError, VarIntPrefixedString,
};
use protocol_version::protocol_version::ProtocolVersion;

impl EncodePacket for String {
    fn encode(
        &self,
        writer: &mut BinaryWriter,
        _protocol_version: ProtocolVersion,
    ) -> Result<(), BinaryWriterError> {
        let protocol_string = VarIntPrefixedString::string(self);
        writer.write(&protocol_string)
    }
}

impl DecodePacket for String {
    fn decode(
        reader: &mut BinaryReader,
        _protocol_version: ProtocolVersion,
    ) -> Result<Self, BinaryReaderError> {
        Ok(reader.read::<VarIntPrefixedString>()?.into_inner())
    }
}
