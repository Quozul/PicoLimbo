use minecraft_protocol::prelude::*;

#[derive(Debug, PacketOut)]
#[packet_id("play/clientbound/minecraft:player_position")]
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

impl Default for SynchronizePlayerPositionPacket {
    fn default() -> Self {
        Self {
            v_1_21_2_teleport_id: VarInt::default(),
            x: 0.0,
            // For 1.19+ we need to spawn player outside the world to avoid stuck in terrain loading
            y: 384.0,
            z: 0.0,
            velocity_x: 0.0,
            velocity_y: 0.0,
            velocity_z: 0.0,
            yaw: 0.0,
            pitch: 0.0,
            v_1_21_2_flags: 0x08,
            flags: 0,
            teleport_id: VarInt::default(),
            dismount_vehicle: false,
            on_ground: false,
        }
    }
}
