use minecraft_protocol::prelude::*;

/// This packet is currently only implemented for 1.7.2 and 1.7.6 as this is a way to ensure the player has spawned into the world.
/// Also, this packet is intended to be used during the play state, putting it in the login state is a hack to make this work.
/// This is because this server will send the keep alive packet before the client is ready to receive it.
#[derive(Debug, PacketIn)]
#[packet_id("login/serverbound/minecraft:player_position")]
pub struct PlayerPositionPacket {
    pub x: f64,
    pub feet_y: f64,
    pub head_y: f64,
    pub z: f64,
    pub on_ground: bool,
}
