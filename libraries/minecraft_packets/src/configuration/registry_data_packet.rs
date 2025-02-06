use crate::configuration::data::registry_entry::RegistryEntry;
use data_types::prelude::{EncodePacketField, Identifier, LengthPaddedVec};
use macros::{PacketOut, packet_id};

#[derive(Debug, PacketOut)]
#[packet_id(0x07)]
pub struct RegistryDataPacket {
    pub registry_id: Identifier,
    pub entries: LengthPaddedVec<RegistryEntry>,
}
