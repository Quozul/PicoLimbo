use data_types::prelude::*;
use macro_traits::prelude::*;
use macros::{PacketIn, packet_id};
use protocol_version::ProtocolVersion::V1_21_4;

#[derive(Debug, PacketIn)]
#[packet_id(0x00)]
pub struct LoginStartPacket {
    pub name: String,
    #[protocol(V1_21_4..)]
    pub player_uuid: Uuid,
}
