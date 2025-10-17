use minecraft_protocol::prelude::*;

#[derive(PacketOut)]
pub struct SynchronizePlayerPositionPacket {
    #[pvn(768..)]
    pub v_1_21_2_teleport_id: VarInt,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    #[pvn(768..)]
    pub velocity_x: f64,
    #[pvn(768..)]
    pub velocity_y: f64,
    #[pvn(768..)]
    pub velocity_z: f64,
    pub yaw: f32,
    pub pitch: f32,
    /// X = 0x01,
    /// Y = 0x02,
    /// Z = 0x04,
    /// Yaw = 0x08,
    /// Pitch = 0x10,
    #[pvn(768..)]
    pub v_1_21_2_flags: i32,
    #[pvn(47..768)]
    pub flags: u8,
    #[pvn(..47)]
    pub on_ground: bool,
    #[pvn(107..768)]
    pub teleport_id: VarInt,
    /// True if the player should dismount their vehicle.
    #[pvn(755..762)]
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
