use thiserror::Error;
use tokio::io::{AsyncRead, AsyncReadExt};

#[async_trait::async_trait]
pub trait DecodePacketField: Sized {
    type Error: std::error::Error;

    /// Decodes an instance of Self from the provided asynchronous reader.
    async fn decode<R>(reader: &mut R) -> Result<Self, Self::Error>
    where
        R: AsyncRead + Unpin + Send;
}

#[derive(Debug, Error)]
pub enum DeserializeNumberError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

macro_rules! impl_deserialize_packet_data {
    ($($t:ty),*) => {
        $(
            #[async_trait::async_trait]
            impl DecodePacketField for $t {
                type Error = DeserializeNumberError;

                async fn decode<T>(reader: &mut T) -> Result<Self, Self::Error>
                where
                    T: AsyncRead + Unpin + Send,
                {
                    let mut buf = [0u8; std::mem::size_of::<$t>()];
                    reader
                        .read_exact(&mut buf)
                        .await?;
                    Ok(<$t>::from_be_bytes(buf))
                }
            }
        )*
    };
}

impl_deserialize_packet_data!(i64, i32, f32, f64, i8, u16, u8);

#[async_trait::async_trait]
impl DecodePacketField for bool {
    type Error = std::io::Error;

    async fn decode<T>(reader: &mut T) -> Result<Self, Self::Error>
    where
        T: AsyncRead + Unpin + Send,
    {
        let mut buf = [0u8; 1];
        reader.read_exact(&mut buf).await?;
        Ok(buf[0] == 0x01)
    }
}
