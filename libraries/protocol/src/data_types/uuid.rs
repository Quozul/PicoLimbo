use crate::prelude::EncodePacketField;
use crate::traits::decode_packet_field::DecodePacketField;
use tokio::io::{AsyncRead, AsyncReadExt};
use uuid::Uuid;

#[async_trait::async_trait]
impl DecodePacketField for Uuid {
    type Error = std::io::Error;

    async fn decode<R>(reader: &mut R) -> Result<Self, Self::Error>
    where
        R: AsyncRead + Unpin + Send,
    {
        let mut data = [0u8; 16];
        reader.read_exact(&mut data).await?;
        Ok(Uuid::from_bytes(data))
    }
}

impl EncodePacketField for Uuid {
    type Error = std::convert::Infallible;

    fn encode(&self, bytes: &mut Vec<u8>) -> Result<(), Self::Error> {
        bytes.extend_from_slice(self.as_bytes());
        Ok(())
    }
}
