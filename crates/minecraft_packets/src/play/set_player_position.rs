use minecraft_protocol::prelude::*;

#[derive(PacketIn)]
pub struct SetPlayerPositionPacket {
    pub x: f64,
    pub feet_y: f64,
    pub z: f64,
    pub flags: u8,
}

impl SetPlayerPositionPacket {
    pub fn position(&self) -> (f64, f64, f64) {
        (self.x, self.feet_y, self.z)
    }
}