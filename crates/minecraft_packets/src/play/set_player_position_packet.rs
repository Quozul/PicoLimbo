use minecraft_protocol::prelude::*;

#[derive(PacketIn)]
pub struct SetPlayerPositionPacket {
    pub x: f64,
    pub feet_y: f64,
    pub z: f64,
    #[protocol_version(min = V1_21_4)]
    pub v1_21_4_flags: u8,
    #[protocol_version(max = V1_21_2)]
    pub on_ground: bool,
}

impl SetPlayerPositionPacket {
    pub fn position(&self) -> (f64, f64, f64) {
        (self.x, self.feet_y, self.z)
    }
}
