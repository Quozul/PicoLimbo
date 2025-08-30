use crate::prelude::{DecodePacket, EncodePacket};
use pico_binutils::prelude::{
    BinaryReader, BinaryReaderError, BinaryWriter, BinaryWriterError, VarInt,
};
use protocol_version::protocol_version::ProtocolVersion;
use std::collections::HashMap;
use std::hash::Hash;

impl<K, V> EncodePacket for HashMap<K, V>
where
    K: EncodePacket + Eq + Hash,
    V: EncodePacket,
{
    fn encode(
        &self,
        writer: &mut BinaryWriter,
        protocol_version: ProtocolVersion,
    ) -> Result<(), BinaryWriterError> {
        let length = VarInt::try_from(self.len())?;
        length.encode(writer, protocol_version)?;
        for (key, value) in self {
            key.encode(writer, protocol_version)?;
            value.encode(writer, protocol_version)?;
        }
        Ok(())
    }
}

impl<K, V> DecodePacket for HashMap<K, V>
where
    K: DecodePacket + Eq + Hash,
    V: DecodePacket,
{
    fn decode(
        reader: &mut BinaryReader,
        protocol_version: ProtocolVersion,
    ) -> Result<Self, BinaryReaderError> {
        let len = VarInt::decode(reader, protocol_version)?;
        let mut hash_map = HashMap::<K, V>::with_capacity(len.inner() as usize);
        for _ in 0..len.inner() {
            let key = K::decode(reader, protocol_version)?;
            let value = V::decode(reader, protocol_version)?;
            hash_map.insert(key, value);
        }
        Ok(hash_map)
    }
}
