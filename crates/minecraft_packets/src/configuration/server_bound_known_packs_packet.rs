use crate::configuration::data::known_pack::KnownPack;
use minecraft_protocol::prelude::*;

#[derive(PacketIn)]
pub struct ServerBoundKnownPacksPacket {
    pub known_packs: LengthPaddedVec<KnownPack>,
}
