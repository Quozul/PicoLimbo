use crate::prelude::{
    BinaryWriter, BinaryWriterError, EncodePacket, Identifier, Optional, ProtocolVersion,
};
use macros::PacketOut;
use pico_nbt::prelude::Nbt;

#[derive(Debug, PacketOut)]
pub struct RegistryEntry {
    pub entry_id: Identifier,
    /// Whether the entry has any data following.
    pub has_data: bool,
    /// Entry data. Only present if Has Data is true.
    pub nbt: Optional<Nbt>,
}
