use minecraft_protocol::prelude::*;

/// This packet was introduced in 1.20.2
/// This packet changes the state to Configuration.
#[derive(Default, PacketIn)]
pub struct LoginAcknowledgedPacket {}
