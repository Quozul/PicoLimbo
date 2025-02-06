use data_types::prelude::DecodePacketField;
use macros::{PacketIn, packet_id};

#[derive(Debug, PacketIn)]
#[packet_id(0x01)]
pub struct PingRequestPacket {
    pub timestamp: i64,
}
