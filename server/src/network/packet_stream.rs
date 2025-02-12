use crate::get_packet_length::{get_packet_length, PacketLengthParseError, MAXIMUM_PACKET_LENGTH};
use crate::network::raw_packet::RawPacket;
use protocol::prelude::*;
use std::convert::Infallible;
use thiserror::Error;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

pub struct PacketStream<Stream>
where
    Stream: AsyncWrite + AsyncRead + Unpin + Send,
{
    stream: Stream,
}

impl<Stream> PacketStream<Stream>
where
    Stream: AsyncWrite + AsyncRead + Unpin + Send,
{
    pub fn new(stream: Stream) -> PacketStream<Stream> {
        PacketStream { stream }
    }

    pub async fn read_packet(&mut self) -> Result<RawPacket, PacketStreamError> {
        let packet_length = get_packet_length(&mut self.stream).await?;
        let mut data = vec![0u8; packet_length];
        self.stream.read_exact(&mut data).await?;
        Ok(RawPacket::new(data))
    }

    pub async fn write_packet(&mut self, packet: RawPacket) -> Result<(), PacketStreamError> {
        let packet_length = packet.size();
        if packet_length > MAXIMUM_PACKET_LENGTH {
            Err(PacketLengthParseError::PacketTooLarge)?;
        }

        let mut var_int_bytes = Vec::new();
        let var_int = VarInt::new(packet_length as i32);
        var_int.encode(&mut var_int_bytes)?;

        self.stream.write_all(&var_int_bytes).await?;
        self.stream.write_all(&[packet.packet_id()]).await?;
        self.stream.write_all(packet.data()).await?;
        self.stream.flush().await?;
        Ok(())
    }
}

#[derive(Error, Debug)]
pub enum PacketStreamError {
    #[error(transparent)]
    IoError(#[from] tokio::io::Error),
    #[error(transparent)]
    VarInt(#[from] PacketLengthParseError),
    #[error(transparent)]
    Infallible(#[from] Infallible),
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_read_simple_packet() {
        // Given
        let reader = tokio_test::io::Builder::new().read(&[1, 42]).build();

        let mut packet_stream = PacketStream::new(reader);

        // When
        let packet = packet_stream.read_packet().await.unwrap();

        // Then
        assert_eq!(packet.size(), 1);
        assert_eq!(packet.packet_id(), 42);
    }

    #[tokio::test]
    async fn test_read_eof() {
        // Given
        let reader = tokio_test::io::Builder::new().read(&[1]).build();

        let mut packet_stream = PacketStream::new(reader);

        // When
        let packet = packet_stream.read_packet().await;

        // Then
        assert!(packet.is_err());
        assert!(matches!(packet.unwrap_err(), PacketStreamError::IoError(_)));
    }

    #[tokio::test]
    async fn test_slow_read_packet() {
        // Given
        let reader = tokio_test::io::Builder::new()
            .read(&[2])
            .wait(Duration::from_millis(10))
            .read(&[42])
            .wait(Duration::from_millis(10))
            .read(&[84])
            .build();

        let mut packet_stream = PacketStream::new(reader);

        // When
        let packet = packet_stream.read_packet().await.unwrap();

        // Then
        assert_eq!(packet.size(), 2);
        assert_eq!(packet.packet_id(), 42);
        assert_eq!(packet.data(), [84]);
    }

    #[tokio::test]
    async fn test_two_packets() {
        // Given
        let reader = tokio_test::io::Builder::new()
            .read(&[1, 42])
            .read(&[2, 42, 84])
            .build();

        let mut packet_stream = PacketStream::new(reader);

        // When
        let packet_1 = packet_stream.read_packet().await.unwrap();
        let packet_2 = packet_stream.read_packet().await.unwrap();

        // Then
        assert_eq!(packet_1.size(), 1);
        assert_eq!(packet_1.packet_id(), 42);

        assert_eq!(packet_2.size(), 2);
        assert_eq!(packet_2.packet_id(), 42);
        assert_eq!(packet_2.data(), [84]);
    }

    // Write tests
    #[tokio::test]
    async fn test_write_simple_packet() {
        // Given
        let packet = RawPacket::new(vec![42, 84]);
        let expected_bytes = vec![2, 42, 84];

        let stream = tokio_test::io::Builder::new()
            .write(&expected_bytes)
            .build();

        let mut packet_stream = PacketStream::new(stream);

        // When / Then
        packet_stream.write_packet(packet).await.unwrap();
    }
}
