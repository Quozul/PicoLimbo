use crate::login::Property;
use minecraft_protocol::prelude::*;

/// This packet was introduced in 1.21.2, previous versions uses the GameProfilePacket.
#[derive(PacketOut)]
pub struct LoginFinishedPacket {
    uuid: UuidAsString,
    username: String,
    #[protocol_version(min = V1_16)]
    properties: LengthPaddedVec<Property>,
    #[protocol_version(min = V26_2)]
    session_id: UuidAsLongs,
}

impl LoginFinishedPacket {
    pub fn new(uuid: Uuid, username: impl ToString) -> Self {
        Self {
            uuid: uuid.into(),
            username: username.to_string(),
            properties: LengthPaddedVec::default(),
            session_id: Uuid::new_v4().into(),
        }
    }
}
