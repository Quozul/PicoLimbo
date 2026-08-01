use minecraft_protocol::prelude::*;

#[derive(PacketOut)]
pub struct SynchronizePlayerPositionPacket {
    #[protocol_version(min = V1_21_2)]
    pub v_1_21_2_teleport_id: VarInt,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    #[protocol_version(min = V1_21_2)]
    pub velocity_x: f64,
    #[protocol_version(min = V1_21_2)]
    pub velocity_y: f64,
    #[protocol_version(min = V1_21_2)]
    pub velocity_z: f64,
    pub yaw: f32,
    pub pitch: f32,
    /// X = 0x01,
    /// Y = 0x02,
    /// Z = 0x04,
    /// Yaw = 0x08,
    /// Pitch = 0x10,
    #[protocol_version(min = V1_21_2)]
    pub v_1_21_2_flags: i32,
    #[protocol_version(min = V1_8, max = V1_21)]
    pub flags: u8,
    #[protocol_version(max = V1_7_6)]
    pub on_ground: bool,
    #[protocol_version(min = V1_9, max = V1_21)]
    pub teleport_id: VarInt,
    /// True if the player should dismount their vehicle.
    #[protocol_version(min = V1_17, max = V1_19_3)]
    pub dismount_vehicle: bool,
}

impl SynchronizePlayerPositionPacket {
    pub fn new(x: f64, y: f64, z: f64, yaw: f32, pitch: f32) -> Self {
        Self {
            v_1_21_2_teleport_id: VarInt::default(),
            x,
            // For 1.19+ we need to spawn player outside the world to avoid stuck in terrain loading
            y,
            z,
            velocity_x: 0.0,
            velocity_y: 0.0,
            velocity_z: 0.0,
            yaw,
            pitch,
            v_1_21_2_flags: 0x00,
            flags: 0,
            teleport_id: VarInt::default(),
            dismount_vehicle: false,
            on_ground: false,
        }
    }
}
