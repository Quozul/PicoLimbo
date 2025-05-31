use minecraft_protocol::prelude::*;

/// This packet was introduced in 1.20.2
/// This packet changes the state to Configuration.
#[derive(Debug, PacketIn)]
#[packet_id("login/serverbound/minecraft:login_acknowledged")]
pub struct LoginAcknowledgedPacket {}
