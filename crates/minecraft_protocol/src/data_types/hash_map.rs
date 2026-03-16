use crate::prelude::EncodePacket;
use pico_binutils::prelude::{BinaryWriter, BinaryWriterError, VarInt};
use protocol_version::protocol_version::ProtocolVersion;
use std::collections::HashMap;

impl<K, V> EncodePacket for HashMap<K, V>
where
    K: EncodePacket,
    V: EncodePacket,
{
    fn encode(
        &self,
        writer: &mut BinaryWriter,
        protocol_version: ProtocolVersion,
    ) -> Result<(), BinaryWriterError> {
        let map_size = VarInt::new(self.len() as i32);
        map_size.encode(writer, protocol_version)?;
        for (key, value) in self {
            key.encode(writer, protocol_version)?;
            value.encode(writer, protocol_version)?;
        }
        Ok(())
    }
}
