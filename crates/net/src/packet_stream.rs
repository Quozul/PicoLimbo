use crate::get_packet_length::{MAXIMUM_PACKET_LENGTH, PacketLengthParseError, get_packet_length};
use crate::raw_packet::RawPacket;
use minecraft_protocol::prelude::*;
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
        let packet_length = self.read_packet_length().await?;

        if packet_length == 0 {
            return Err(PacketStreamError::EmptyPacket);
        }

        let mut data = vec![0u8; packet_length];
        self.stream.read_exact(&mut data).await?;

        RawPacket::new(data).map_err(|_| PacketStreamError::EmptyPacket)
    }

    pub async fn write_packet(&mut self, packet: RawPacket) -> Result<(), PacketStreamError> {
        let packet_length = packet.size();
        if packet_length > MAXIMUM_PACKET_LENGTH {
            Err(PacketLengthParseError::PacketTooLarge)?;
        }

        let var_int = VarInt::new(packet_length as i32);
        let mut writer = BinaryWriter::new();
        var_int
            .encode(&mut writer, ProtocolVersion::default())
            .map_err(PacketStreamError::BinaryWriter)?;
        let var_int_bytes = writer.into_inner();

        if let Some(packet_id) = packet.packet_id() {
            self.stream.write_all(&var_int_bytes).await?;
            self.stream.write_u8(packet_id).await?;
            self.stream.write_all(packet.data()).await?;
            self.stream.flush().await?;
            Ok(())
        } else {
            Err(PacketStreamError::MissingPacketId)
        }
    }

    pub fn get_stream(&mut self) -> &mut Stream {
        &mut self.stream
    }

    async fn read_packet_length(&mut self) -> Result<usize, PacketStreamError> {
        let mut var_int_buf = Vec::new();

        for _ in 0..5 {
            let mut byte = [0u8; 1];
            self.stream.read_exact(&mut byte).await?;
            var_int_buf.push(byte[0]);

            match get_packet_length(&var_int_buf) {
                Ok(length) => return Ok(length),
                Err(PacketLengthParseError::BinaryReader(BinaryReaderError::UnexpectedEof)) => {
                    continue;
                }
                Err(e) => return Err(e.into()),
            }
        }

        Err(PacketLengthParseError::BinaryReader(BinaryReaderError::UnexpectedEof).into())
    }
}

#[derive(Error, Debug)]
pub enum PacketStreamError {
    #[error(transparent)]
    IoError(#[from] tokio::io::Error),
    #[error(transparent)]
    VarInt(#[from] PacketLengthParseError),
    #[error("empty packet")]
    EmptyPacket,
    #[error("missing packet id")]
    MissingPacketId,
    #[error("binary writer")]
    BinaryWriter(BinaryWriterError),
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
        assert_eq!(packet.packet_id().unwrap(), 42);
    }

    #[tokio::test]
    async fn test_read_empty_packet() {
        // Given
        let reader = tokio_test::io::Builder::new().read(&[0]).build();

        let mut packet_stream = PacketStream::new(reader);

        // When
        let packet = packet_stream.read_packet().await;

        // Then
        assert!(packet.is_err());
        assert!(matches!(
            packet.unwrap_err(),
            PacketStreamError::EmptyPacket
        ));
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
        assert_eq!(packet.packet_id().unwrap(), 42);
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
        assert_eq!(packet_1.packet_id().unwrap(), 42);

        assert_eq!(packet_2.size(), 2);
        assert_eq!(packet_2.packet_id().unwrap(), 42);
        assert_eq!(packet_2.data(), [84]);
    }

    // Write tests
    #[tokio::test]
    async fn test_write_simple_packet() {
        // Given
        let packet = RawPacket::new(vec![42, 84]).unwrap();
        let expected_bytes = vec![2, 42, 84];

        let stream = tokio_test::io::Builder::new()
            .write(&expected_bytes)
            .build();

        let mut packet_stream = PacketStream::new(stream);

        // When / Then
        packet_stream.write_packet(packet).await.unwrap();
    }
}
