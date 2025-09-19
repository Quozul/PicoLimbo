use minecraft_protocol::prelude::*;

/// Max protocol version for this is 578 or 1.15.5 included
#[derive(PacketOut)]
pub struct PreV1_16Data {
    pub game_mode: u8,
    pub dimension: DimensionField,
    #[pvn(573..)]
    pub v1_15_hashed_seed: i64,
    #[pvn(..477)]
    pub difficulty: u8,
    pub max_players: u8,
    pub level_type: String,
    #[pvn(477..)]
    pub v1_14_view_distance: VarInt,
    #[pvn(47..)]
    pub v1_8_reduced_debug_info: bool,
    #[pvn(573..)]
    pub v1_15_enable_respawn_screen: bool,
}

impl Default for PreV1_16Data {
    fn default() -> Self {
        Self {
            game_mode: 3,
            max_players: 1,
            level_type: "default".to_string(),
            v1_14_view_distance: VarInt::new(10),
            v1_8_reduced_debug_info: false,
            v1_15_enable_respawn_screen: true,
            dimension: DimensionField(0),
            v1_15_hashed_seed: 0,
            difficulty: 0,
        }
    }
}

pub struct DimensionField(pub i8);

impl EncodePacket for DimensionField {
    fn encode(
        &self,
        writer: &mut BinaryWriter,
        protocol_version: ProtocolVersion,
    ) -> Result<(), BinaryWriterError> {
        if protocol_version.is_after_inclusive(ProtocolVersion::V1_9_1) {
            i32::from(self.0).encode(writer, protocol_version)
        } else {
            self.0.encode(writer, protocol_version)
        }
    }
}
