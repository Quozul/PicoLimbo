use crate::configuration::data::known_pack::KnownPack;
use data_types::prelude::{EncodePacketField, LengthPaddedVec};
use macros::{PacketOut, packet_id};

#[derive(Debug, PacketOut)]
#[packet_id(0x0E)]
pub struct ClientBoundKnownPacksPacket {
    known_packs: LengthPaddedVec<KnownPack>,
}

impl Default for ClientBoundKnownPacksPacket {
    fn default() -> Self {
        Self {
            known_packs: vec![KnownPack::default()].into(),
        }
    }
}
