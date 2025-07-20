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
            properties: LengthPaddedVec::default(),
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

impl EncodePacket for Property {
    fn encode(
        &self,
        writer: &mut BinaryWriter,
        protocol_version: ProtocolVersion,
    ) -> Result<(), BinaryWriterError> {
        self.name.encode(writer, protocol_version)?;
        self.value.encode(writer, protocol_version)?;
        self.is_signed.encode(writer, protocol_version)?;
        if let Some(signature) = &self.signature {
            signature.encode(writer, protocol_version)?;
        }
        Ok(())
    }
}
