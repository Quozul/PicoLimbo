use minecraft_protocol::data::registry::registry_entry::RegistryEntry;
use minecraft_protocol::prelude::*;

/// This packet is to use with >= 1.20.5
#[derive(Debug, PacketOut)]
#[packet_id("configuration/clientbound/minecraft:registry_data")]
pub struct RegistryDataPacket {
    pub registry_id: Identifier,
    pub entries: LengthPaddedVec<RegistryEntry>,
}

/// This packet is to use with 1.20.2 < 1.20.5
#[derive(Debug, PacketOut)]
#[packet_id("configuration/clientbound/minecraft:registry_data")]
pub struct RegistryDataCodecPacket {
    pub registry_codec: Nbt,
}
