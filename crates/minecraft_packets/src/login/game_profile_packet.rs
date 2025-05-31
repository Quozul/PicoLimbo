use crate::login::login_success_packet::Property;
use minecraft_protocol::prelude::*;

/// This is the equivalent of LoginSuccessPacket but for versions before 1.21.2.
/// For versions prior to 1.20.2, this packet changes the state to Play.
#[derive(Debug, PacketOut)]
#[packet_id("login/clientbound/minecraft:game_profile")]
pub struct GameProfilePacket {
    #[pvn(735..)]
    v1_16_uuid: Uuid,
    #[pvn(..735)]
    uuid: String,
    username: String,
    #[pvn(759..)]
    properties: LengthPaddedVec<Property>,
    #[pvn(766..)]
    strict_error_handling: bool,
}

impl GameProfilePacket {
    pub fn new(uuid: Uuid, username: impl ToString) -> Self {
        Self {
            v1_16_uuid: uuid,
            uuid: uuid.to_string(),
            username: username.to_string(),
            properties: Vec::new().into(),
            strict_error_handling: false,
        }
    }
}
