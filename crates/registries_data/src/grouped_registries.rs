use minecraft_protocol::prelude::*;

#[derive(PacketIn, PacketOut)]
pub struct V1_20_5RegistryEntry {
    pub entry_id: Identifier,
    pub nbt_bytes: LengthPaddedVec<u8>,
}

#[derive(PacketIn, PacketOut)]
pub struct V1_20_5RegistryEntries {
    pub registry_id: Identifier,
    pub entries: LengthPaddedVec<V1_20_5RegistryEntry>,
}

#[derive(PacketIn, PacketOut)]
pub struct V1_20_5Registries {
    pub registries: LengthPaddedVec<V1_20_5RegistryEntries>,
}
