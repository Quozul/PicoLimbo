use minecraft_protocol::prelude::*;

#[derive(PacketOut)]
pub struct UpdateTagsPacket {
    registry_id: Identifier,
    tags: LengthPaddedVec<RegistryTag>,
}

#[derive(PacketOut)]
struct RegistryTag {
    identifier: Identifier,
    ids: LengthPaddedVec<VarInt>,
}
