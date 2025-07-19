use minecraft_protocol::prelude::*;

/// This packet was introduced in 1.21.2, previous versions uses the GameProfilePacket.
#[derive(Debug, PacketOut)]
#[packet_id("login/clientbound/minecraft:login_finished")]
pub struct LoginSuccessPacket {
    uuid: Uuid,
    username: String,
    #[pvn(735..)]
    properties: LengthPaddedVec<Property>,
}

impl LoginSuccessPacket {
    pub fn new(uuid: Uuid, username: impl ToString) -> Self {
        Self {
            uuid,
            username: username.to_string(),
            properties: Vec::new().into(),
        }
    }
}

#[derive(Debug)]
pub struct Property {
    pub name: String,
    pub value: String,
    pub is_signed: bool,
    pub signature: Option<String>,
}

impl EncodePacketField for Property {
    type Error = std::convert::Infallible;

    fn encode(&self, bytes: &mut Vec<u8>, protocol_version: i32) -> Result<(), Self::Error> {
        self.name.encode(bytes, protocol_version)?;
        self.value.encode(bytes, protocol_version)?;
        self.is_signed.encode(bytes, protocol_version)?;
        if let Some(signature) = &self.signature {
            signature.encode(bytes, protocol_version)?;
        }
        Ok(())
    }
}
