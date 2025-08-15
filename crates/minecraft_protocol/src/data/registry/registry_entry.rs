use crate::prelude::*;
use pico_binutils::prelude::{BinaryWriter, BinaryWriterError};
use protocol_version::protocol_version::ProtocolVersion;

#[derive(Debug)]
pub struct RegistryEntry {
    pub entry_id: Identifier,
    /// Whether the entry has any data following.
    pub has_data: bool,
    /// Entry data. Only present if Has Data is true.
    pub nbt: Option<Nbt>,
}

impl EncodePacket for RegistryEntry {
    fn encode(
        &self,
        writer: &mut BinaryWriter,
        protocol_version: ProtocolVersion,
    ) -> Result<(), BinaryWriterError> {
        self.entry_id.encode(writer, protocol_version)?;
        self.has_data.encode(writer, protocol_version)?;

        if let Some(nbt) = &self.nbt {
            nbt.encode(writer, protocol_version)?;
        }
        Ok(())
    }
}
