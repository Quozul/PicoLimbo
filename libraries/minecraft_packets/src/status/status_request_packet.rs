use macros::{PacketIn, packet_id};

#[derive(Debug, PacketIn)]
#[packet_id(0x00)]
pub struct StatusRequestPacket {}
