use crate::configuration::data::known_pack::KnownPack;
use minecraft_protocol::prelude::*;

#[derive(Debug, PacketOut)]
pub struct ClientBoundKnownPacksPacket {
    pub known_packs: LengthPaddedVec<KnownPack>,
}

impl Default for ClientBoundKnownPacksPacket {
    fn default() -> Self {
        Self {
            known_packs: LengthPaddedVec::new(vec![KnownPack::default()]),
        }
    }
}
