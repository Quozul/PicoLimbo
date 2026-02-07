use crate::prelude::EncodePacket;
use pico_binutils::prelude::{BinaryWriter, BinaryWriterError};
use pico_nbt2::{NbtOptions, Value};
use protocol_version::protocol_version::ProtocolVersion;

impl EncodePacket for Value {
    fn encode(
        &self,
        writer: &mut BinaryWriter,
        protocol_version: ProtocolVersion,
    ) -> Result<(), BinaryWriterError> {
        let nbt_bytes =
            pico_nbt2::to_bytes_with_options(self, None, from_protocol_version(protocol_version))
                .map_err(|_| BinaryWriterError::UnsupportedOperation)?;
        writer.write_bytes(&nbt_bytes)?;
        Ok(())
    }
}

fn from_protocol_version(value: ProtocolVersion) -> NbtOptions {
    NbtOptions::new()
        .nameless_root(value.is_after_inclusive(ProtocolVersion::V1_20_2))
        .dynamic_lists(value.is_after_inclusive(ProtocolVersion::V1_21_5))
}
