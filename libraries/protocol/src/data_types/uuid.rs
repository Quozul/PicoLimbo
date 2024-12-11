use crate::deserialize_packet::DeserializePacketData;
use crate::prelude::SerializePacketData;
use uuid::Uuid;

impl DeserializePacketData for Uuid {
    type Error = std::convert::Infallible;

    fn decode(bytes: &[u8], index: &mut usize) -> Result<Self, Self::Error> {
        let mut data = [0u8; 16];
        data.copy_from_slice(&bytes[*index..*index + 16]);
        *index += 16;
        Ok(Uuid::from_bytes(data))
    }
}

impl SerializePacketData for Uuid {
    type Error = std::convert::Infallible;

    fn encode(&self, bytes: &mut Vec<u8>) -> Result<(), Self::Error> {
        bytes.extend_from_slice(self.as_bytes());
        Ok(())
    }
}
