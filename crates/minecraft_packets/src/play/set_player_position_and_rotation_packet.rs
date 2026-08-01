use minecraft_protocol::prelude::*;

#[derive(PacketIn)]
pub struct SetPlayerPositionAndRotationPacket {
    pub x: f64,
    pub feet_y: f64,
    pub z: f64,
    pub yaw: f32,
    pub pitch: f32,
    #[protocol_version(min = V1_21_4)]
    pub v1_21_4_flags: u8,
    #[protocol_version(max = V1_21_2)]
    pub on_ground: bool,
}
