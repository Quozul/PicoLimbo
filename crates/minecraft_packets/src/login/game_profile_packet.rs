use crate::login::Property;
use minecraft_protocol::prelude::*;

/// This is the equivalent of LoginSuccessPacket but for versions before 1.21.2.
/// For versions prior to 1.20.2, this packet changes the state to Play.
#[derive(PacketOut)]
pub struct GameProfilePacket {
    uuid: UuidAsString,
    username: String,
    #[pvn(759..)]
    properties: LengthPaddedVec<Property>,
    #[pvn(766..)]
    strict_error_handling: bool,
}

impl GameProfilePacket {
    pub fn new(uuid: Uuid, username: impl ToString) -> Self {
        Self {
            uuid: uuid.into(),
            username: username.to_string(),
            properties: LengthPaddedVec::default(),
            strict_error_handling: false,
        }
    }
}
