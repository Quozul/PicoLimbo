use minecraft_protocol::prelude::*;

#[derive(Eq, PartialEq, Clone, PacketOut, PacketIn, Hash, Ord, PartialOrd)]
pub struct InternalProperties {
    pub name: String,
    pub value: String,
}

#[derive(Eq, PartialEq, PacketOut, PacketIn, Hash)]
pub struct InternalState {
    pub internal_id: InternalId,
    pub properties: LengthPaddedVec<InternalProperties>,
}

#[derive(Eq, PartialEq, PacketOut, PacketIn, Hash)]
pub struct InternalBlockMapping {
    pub name: String,
    pub states: LengthPaddedVec<InternalState>,
    pub default_internal_id: InternalId,
}

#[derive(PacketOut, PacketIn)]
pub struct InternalMapping {
    pub mapping: LengthPaddedVec<InternalBlockMapping>,
}

pub type InternalId = u16;
