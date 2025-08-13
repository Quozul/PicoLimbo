use minecraft_protocol::prelude::*;

#[derive(Debug, PacketIn)]
#[packet_id("status/serverbound/minecraft:status_request")]
pub struct StatusRequestPacket {}
