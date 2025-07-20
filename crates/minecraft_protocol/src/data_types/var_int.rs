use crate::prelude::{DecodePacket, EncodePacket};
use crate::protocol_version::ProtocolVersion;
use pico_binutils::prelude::{
    BinaryReader, BinaryReaderError, BinaryWriter, BinaryWriterError, VarInt,
};

impl DecodePacket for VarInt {
    fn decode(
        reader: &mut BinaryReader,
        _protocol_version: ProtocolVersion,
    ) -> Result<Self, BinaryReaderError> {
        reader.read()
    }
}

impl EncodePacket for VarInt {
    fn encode(
        &self,
        writer: &mut BinaryWriter,
        _protocol_version: ProtocolVersion,
    ) -> Result<(), BinaryWriterError> {
        writer.write(self)
    }
}
