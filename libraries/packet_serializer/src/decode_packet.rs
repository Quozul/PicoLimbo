use crate::decode_packet_error::DecodePacketError;
use tokio::io::AsyncRead;

#[async_trait::async_trait]
pub trait DecodePacket: Sized {
    async fn decode<R>(reader: &mut R, protocol_version: u32) -> Result<Self, DecodePacketError>
    where
        R: AsyncRead + Unpin + Send;
}
