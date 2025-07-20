use crate::prelude::EncodePacket;
use crate::protocol_version::ProtocolVersion;
use nbt::prelude::{Nbt, NbtFeatures};
use pico_binutils::prelude::{BinaryWriter, BinaryWriterError};

impl EncodePacket for Nbt {
    fn encode(
        &self,
        writer: &mut BinaryWriter,
        protocol_version: ProtocolVersion,
    ) -> Result<(), BinaryWriterError> {
        let nbt_bytes = self.to_bytes(protocol_version.into())?;
        writer.write_bytes(&nbt_bytes)?;
        Ok(())
    }
}

impl From<ProtocolVersion> for NbtFeatures {
    fn from(value: ProtocolVersion) -> Self {
        let mut builder = NbtFeatures::builder();
        if value.is_after_inclusive(ProtocolVersion::V1_20_2) {
            builder.nameless();
        };
        if value.is_after_inclusive(ProtocolVersion::V1_21_5) {
            builder.dynamic_lists();
        };
        builder.build()
    }
}
