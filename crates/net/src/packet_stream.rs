use crate::get_packet_length::{MAXIMUM_PACKET_LENGTH, PacketLengthParseError, get_packet_length};
use crate::raw_packet::RawPacket;
use minecraft_protocol::prelude::*;
use minecraft_protocol::protocol_version::ProtocolVersion;
use std::convert::Infallible;
use thiserror::Error;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

pub struct PacketStream<Stream>
where
    Stream: AsyncWrite + AsyncRead + Unpin,
{
    stream: Stream,
}

impl<Stream> PacketStream<Stream>
where
    Stream: AsyncWrite + AsyncRead + Unpin,
{
    pub fn new(stream: Stream) -> PacketStream<Stream> {
        PacketStream { stream }
    }

    pub async fn read_packet(&mut self) -> Result<RawPacket, PacketStreamError> {
        let mut var_int_buf = Vec::new();

        for _ in 0..5 {
            let mut byte = [0u8; 1];
            self.stream.read_exact(&mut byte).await?;
            var_int_buf.push(byte[0]);

            match get_packet_length(&var_int_buf) {
                Ok(packet_length) => {
                    let mut data = vec![0u8; packet_length];
                    self.stream.read_exact(&mut data).await?;
                    return Ok(RawPacket::new(data));
                }
                Err(PacketLengthParseError::IncompleteLength) => {
                    continue;
                }
                Err(e) => Err(e)?,
            }
        }
        Err(PacketLengthParseError::IncompleteLength)?
    }

    pub async fn write_packet(&mut self, packet: RawPacket) -> Result<(), PacketStreamError> {
        let packet_length = packet.size();
        if packet_length > MAXIMUM_PACKET_LENGTH {
            Err(PacketLengthParseError::PacketTooLarge)?;
        }

        let mut var_int_bytes = Vec::new();
        let var_int = VarInt::new(packet_length as i32);
        var_int.encode(
            &mut var_int_bytes,
            ProtocolVersion::default().version_number(),
        )?;

        self.stream.write_all(&var_int_bytes).await?;
        self.stream.write_all(&[packet.packet_id()]).await?;
        self.stream.write_all(packet.data()).await?;
        self.stream.flush().await?;
        Ok(())
    }

    pub fn get_stream(&mut self) -> &mut Stream {
        &mut self.stream
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
