use minecraft_protocol::prelude::*;

#[derive(Debug, Eq, Hash, PartialEq, Ord, PartialOrd, Clone, PacketOut, PacketIn)]
pub struct InternalProperties {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Eq, Hash, PartialEq, Ord, PartialOrd, PacketOut, PacketIn)]
pub struct InternalState {
    pub internal_id: InternalId,
    pub properties: LengthPaddedVec<InternalProperties>,
}

#[derive(Debug, Eq, Hash, PartialEq, Ord, PartialOrd, PacketOut, PacketIn)]
pub struct InternalBlockMapping {
    pub name: String,
    pub states: LengthPaddedVec<InternalState>,
    pub default_internal_id: InternalId,
}

#[derive(Debug, PacketOut, PacketIn)]
pub struct InternalMapping {
    pub mapping: LengthPaddedVec<InternalBlockMapping>,
}

pub type InternalId = u16;
