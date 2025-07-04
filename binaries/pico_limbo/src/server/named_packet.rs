use minecraft_protocol::prelude::{DecodePacket, DecodePacketError};
use minecraft_protocol::protocol_version::ProtocolVersion;

pub struct NamedPacket {
    pub name: String,
    pub data: Vec<u8>,
}

impl NamedPacket {
    pub fn decode<T>(&self, protocol_version: ProtocolVersion) -> Result<T, DecodePacketError>
    where
        T: DecodePacket,
    {
        T::decode(&self.data, protocol_version.version_number())
    }
}
