use crate::configuration::data::known_pack::KnownPack;
use minecraft_protocol::prelude::*;

#[derive(PacketIn)]
pub struct ServerBoundKnownPacksPacket {
    known_packs: LengthPaddedVec<KnownPack>,
}

impl ServerBoundKnownPacksPacket {
    pub fn new(known_packs: Vec<KnownPack>) -> Self {
        Self {
            known_packs: LengthPaddedVec::new(known_packs),
        }
    }

    pub fn has_minecraft_core(&self) -> bool {
        self.known_packs
            .inner()
            .iter()
            .any(KnownPack::is_minecraft_core)
    }
}
