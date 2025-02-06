use data_types::prelude::EncodePacketField;
use macros::{PacketOut, packet_id};

#[derive(Debug, PacketOut)]
#[packet_id(0x01)]
pub struct PingResponsePacket {
    pub timestamp: i64,
}
