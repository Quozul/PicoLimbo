use data_types::prelude::DecodePacketField;
use macros::{PacketIn, packet_id};

#[derive(Debug, PacketIn)]
#[packet_id(0x1A)]
pub struct ServerBoundKeepAlivePacket {
    id: i64,
}
