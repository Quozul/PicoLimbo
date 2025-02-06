use data_types::prelude::DecodePacketField;
use macros::{PacketIn, packet_id};

#[derive(Debug, PacketIn)]
#[packet_id(0x1C)]
pub struct SetPlayerPositionPacket {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    on_ground: bool,
}

#[derive(Debug, PacketIn)]
#[packet_id(0x1D)]
pub struct SetPlayerPositionAndRotationPacket {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub yaw: f32,
    pub pitch: f32,
    on_ground: bool,
}

#[derive(Debug, PacketIn)]
#[packet_id(0x1E)]
pub struct SetPlayerRotationPacket {
    pub yaw: f32,
    pub pitch: f32,
}
