use crate::login::Property;
use minecraft_protocol::prelude::*;

/// This packet was introduced in 1.21.2, previous versions uses the GameProfilePacket.
#[derive(PacketOut)]
pub struct LoginSuccessPacket {
    uuid: UuidAsString,
    username: String,
    #[pvn(735..)]
    properties: LengthPaddedVec<Property>,
}

impl LoginSuccessPacket {
    pub fn new(uuid: Uuid, username: impl ToString) -> Self {
        Self {
            uuid: uuid.into(),
            username: username.to_string(),
            properties: LengthPaddedVec::default(),
        }
    }
}
