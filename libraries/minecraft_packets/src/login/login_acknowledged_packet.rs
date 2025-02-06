use macros::{PacketIn, packet_id};

#[derive(Debug, PacketIn)]
#[packet_id(0x03)]
pub struct LoginAcknowledgedPacket {}
