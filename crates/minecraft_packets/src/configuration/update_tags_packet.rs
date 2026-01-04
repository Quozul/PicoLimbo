use minecraft_protocol::prelude::*;

#[derive(PacketOut, Debug)]
pub struct UpdateTagsPacket {
    tagged_registries: LengthPaddedVec<TaggedRegistry>,
}

impl UpdateTagsPacket {
    pub fn new(tagged_registries: Vec<TaggedRegistry>) -> Self {
        Self {
            tagged_registries: LengthPaddedVec::new(tagged_registries),
        }
    }
}

#[derive(PacketOut, Debug)]
pub struct TaggedRegistry {
    registry_id: Identifier,
    tags: LengthPaddedVec<RegistryTag>,
}

impl TaggedRegistry {
    pub fn new(registry_id: Identifier, tags: Vec<RegistryTag>) -> Self {
        Self {
            registry_id,
            tags: LengthPaddedVec::new(tags),
        }
    }
}

#[derive(PacketOut, Debug)]
pub struct RegistryTag {
    identifier: Identifier,
    ids: LengthPaddedVec<VarInt>,
}

impl RegistryTag {
    pub fn new(identifier: Identifier, ids: Vec<VarInt>) -> Self {
        Self {
            identifier,
            ids: LengthPaddedVec::new(ids),
        }
    }
}
