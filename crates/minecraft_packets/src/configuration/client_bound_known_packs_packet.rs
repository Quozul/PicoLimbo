use crate::configuration::data::known_pack::KnownPack;
use minecraft_protocol::prelude::*;

#[derive(PacketOut)]
pub struct ClientBoundKnownPacksPacket {
    pub known_packs: LengthPaddedVec<KnownPack>,
}

impl ClientBoundKnownPacksPacket {
    pub fn new(version: &str) -> Self {
        Self {
            known_packs: LengthPaddedVec::new(vec![KnownPack::new(version)]),
        }
    }
}
