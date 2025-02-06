use crate::configuration::data::known_pack::KnownPack;
use data_types::prelude::{DecodePacketField, LengthPaddedVec};
use macros::{PacketIn, packet_id};

#[derive(Debug, PacketIn)]
#[packet_id(0x07)]
pub struct ClientKnownPacksPacket {
    known_packs: LengthPaddedVec<KnownPack>,
}

impl Default for ClientKnownPacksPacket {
    fn default() -> Self {
        Self {
            known_packs: vec![KnownPack::default()].into(),
        }
    }
}
