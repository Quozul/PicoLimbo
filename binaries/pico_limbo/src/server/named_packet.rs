use minecraft_protocol::prelude::{BinaryReader, BinaryReaderError, DecodePacket};
use minecraft_protocol::protocol_version::ProtocolVersion;

pub struct NamedPacket {
    pub name: String,
    pub data: Vec<u8>,
}

impl NamedPacket {
    pub fn decode<T>(&self, protocol_version: ProtocolVersion) -> Result<T, BinaryReaderError>
    where
        T: DecodePacket,
    {
        let mut reader = BinaryReader::new(&self.data);
        T::decode(&mut reader, protocol_version)
    }
}
