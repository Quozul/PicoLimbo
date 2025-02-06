use data_types::prelude::{EncodePacketField, VarInt};
use macros::{PacketOut, packet_id};

#[derive(Debug, PacketOut)]
#[packet_id(0x42)]
pub struct SynchronizePlayerPositionPacket {
    pub teleport_id: VarInt,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub unknown_a: f64,
    pub unknown_b: f64,
    pub unknown_c: f64,
    pub yaw: f32,
    pub pitch: f32,
    /// X = 0x01,
    /// Y = 0x02,
    /// Z = 0x04,
    /// Yaw = 0x08,
    /// Pitch = 0x10,
    pub flags: i32,
}

impl Default for SynchronizePlayerPositionPacket {
    fn default() -> Self {
        Self {
            teleport_id: VarInt::default(),
            x: 0.0,
            y: 384.0,
            z: 0.0,
            unknown_a: 0.0,
            unknown_b: 0.0,
            unknown_c: 0.0,
            yaw: 0.0,
            pitch: 0.0,
            flags: 0x08,
        }
    }
}
