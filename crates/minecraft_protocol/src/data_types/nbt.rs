use crate::prelude::EncodePacket;
use pico_binutils::prelude::{BinaryWriter, BinaryWriterError};
use pico_nbt::prelude::{Nbt, NbtFeatures};
use protocol_version::protocol_version::ProtocolVersion;

impl EncodePacket for Nbt {
    fn encode(
        &self,
        writer: &mut BinaryWriter,
        protocol_version: ProtocolVersion,
    ) -> Result<(), BinaryWriterError> {
        let nbt_bytes = self.to_bytes(from_protocol_version(protocol_version))?;
        writer.write_bytes(&nbt_bytes)?;
        Ok(())
    }
}

fn from_protocol_version(value: ProtocolVersion) -> NbtFeatures {
    let mut builder = NbtFeatures::builder();
    if value.is_after_inclusive(ProtocolVersion::V1_20_2) {
        builder.nameless();
    };
    if value.is_after_inclusive(ProtocolVersion::V1_21_5) {
        builder.dynamic_lists();
    };
    builder.build()
}
