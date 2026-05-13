use crate::configuration::data::known_pack::KnownPack;
use minecraft_protocol::prelude::*;

#[derive(PacketOut)]
pub struct ClientBoundKnownPacksPacket {
    pub known_packs: LengthPaddedVec<KnownPack>,
}

impl ClientBoundKnownPacksPacket {
    pub fn new(versions: &[&str]) -> Self {
        let known_packs = versions
            .iter()
            .map(|version| KnownPack::new(version))
            .collect::<Vec<_>>();
        Self {
            known_packs: LengthPaddedVec::new(known_packs),
        }
    }
}
