use protocol::prelude::*;
use thiserror::Error;
use tokio::io::AsyncRead;

#[derive(Debug)]
pub struct KnownPack {
    pub namespace: String,
    pub id: String,
    pub version: String,
}

impl Default for KnownPack {
    fn default() -> Self {
        Self {
            namespace: "minecraft".to_string(),
            id: "core".to_string(),
            version: "1.21.4".to_string(),
        }
    }
}

#[derive(Error, Debug)]
#[error("error while decoding a packet; error={0}")]
pub enum DecodePacketError {
    #[error("error while decoding a string")]
    String(#[from] StringDecodingError),
}

#[async_trait::async_trait]
impl DecodePacketField for KnownPack {
    type Error = DecodePacketError;

    async fn decode<T>(reader: &mut T) -> Result<Self, Self::Error>
    where
        T: AsyncRead + Unpin + Send,
    {
        let namespace = String::decode(reader).await?;
        let id = String::decode(reader).await?;
        let version = String::decode(reader).await?;

        Ok(Self {
            namespace,
            id,
            version,
        })
    }
}

impl EncodePacketField for KnownPack {
    type Error = std::convert::Infallible;

    fn encode(&self, bytes: &mut Vec<u8>) -> Result<(), Self::Error> {
        self.namespace.encode(bytes)?;
        self.id.encode(bytes)?;
        self.version.encode(bytes)?;
        Ok(())
    }
}
