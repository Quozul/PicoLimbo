use data_types::prelude::{EncodePacketField, LengthPaddedVec, Uuid};
use macros::{PacketOut, packet_id};
use protocol_version::ProtocolVersion::{V1_7_2, V1_9, V1_21_2};

#[derive(Debug, PacketOut)]
#[packet_id(0x02)]
pub struct LoginSuccessPacket {
    #[protocol(V1_21_2..)]
    uuid: Uuid,
    #[protocol(V1_7_2..V1_9)]
    uuid_string: String,
    username: String,
    #[protocol(V1_21_2..)]
    properties: LengthPaddedVec<Property>,
}

impl LoginSuccessPacket {
    pub fn new(uuid: Uuid, username: String) -> Self {
        Self {
            uuid,
            uuid_string: uuid.to_string(),
            username,
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

    fn encode(&self, bytes: &mut Vec<u8>) -> Result<(), Self::Error> {
        self.name.encode(bytes)?;
        self.value.encode(bytes)?;
        self.is_signed.encode(bytes)?;
        if let Some(signature) = &self.signature {
            signature.encode(bytes)?;
        }
        Ok(())
    }
}
