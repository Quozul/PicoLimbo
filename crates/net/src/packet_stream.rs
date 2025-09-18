use crate::get_packet_length::{MAXIMUM_PACKET_LENGTH, PacketLengthParseError, get_packet_length};
use crate::raw_packet::RawPacket;
use flate2::Compression;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use minecraft_protocol::prelude::*;
use std::io::{self, Read, Write};
use std::num::TryFromIntError;
use thiserror::Error;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

#[derive(Clone)]
struct CompressionSettings {
    threshold: usize,
    level: Compression,
}

pub struct PacketStream<Stream>
where
    Stream: AsyncWrite + AsyncRead + Unpin,
{
    stream: Stream,
    compression_settings: Option<CompressionSettings>,
}

impl<Stream> PacketStream<Stream>
where
    Stream: AsyncWrite + AsyncRead + Unpin,
{
    /// Creates a new `PacketStream` without compression.
    pub const fn new(stream: Stream) -> Self {
        Self {
            stream,
            compression_settings: None,
        }
    }

    /// Enables or disables compression.
    ///
    /// - `Some(threshold)`: Enables compression for packets with a payload size
    ///   greater than or equal to `threshold`.
    /// - `None`: Disables compression entirely.
    pub fn set_compression(&mut self, threshold: usize, level: u32) {
        self.compression_settings = Some(CompressionSettings {
            threshold,
            level: Compression::new(level.clamp(0, 9)),
        });
    }

    /// Reads a single packet from the stream, handling decompression if enabled.
    pub async fn read_packet(&mut self) -> Result<RawPacket, PacketStreamError> {
        match self.compression_settings {
            None => self.read_uncompressed_packet().await,
            Some(_) => self.read_compressed_packet_format().await,
        }
    }

    /// Writes a single packet to the stream, handling compression if enabled.
    pub async fn write_packet(&mut self, packet: RawPacket) -> Result<(), PacketStreamError> {
        match self.compression_settings() {
            None => self.write_uncompressed_packet(packet).await,
            Some(ref settings) => self.write_compressed_packet_format(packet, settings).await,
        }
    }

    fn compression_settings(&self) -> Option<CompressionSettings> {
        self.compression_settings.clone()
    }

    /// Gets a mutable reference to the underlying stream.
    pub const fn get_stream(&mut self) -> &mut Stream {
        &mut self.stream
    }

    async fn read_uncompressed_packet(&mut self) -> Result<RawPacket, PacketStreamError> {
        let packet_length = self.read_and_validate_packet_length().await?;

        if packet_length == 0 {
            return Err(PacketStreamError::ZeroLengthPacket);
        }

        let mut data = vec![0u8; packet_length];
        self.stream.read_exact(&mut data).await?;

        RawPacket::new(data).map_err(|_| PacketStreamError::MissingPacketId)
    }

    async fn read_compressed_packet_format(&mut self) -> Result<RawPacket, PacketStreamError> {
        let packet_length = self.read_and_validate_packet_length().await?;

        if packet_length == 0 {
            return Err(PacketStreamError::ZeroLengthPacket);
        }

        let mut packet_content_buf = vec![0u8; packet_length];
        self.stream.read_exact(&mut packet_content_buf).await?;

        let mut reader = BinaryReader::new(&packet_content_buf);
        let data_length = usize::try_from(reader.read::<VarInt>()?.inner())?;
        let payload_bytes = reader.remaining_bytes()?;

        let packet_data = if data_length > 0 {
            decompress_data(&payload_bytes, data_length)?
        } else {
            // Data length of 0 means the packet is not compressed.
            payload_bytes
        };

        RawPacket::new(packet_data).map_err(|_| PacketStreamError::MissingPacketId)
    }

    async fn write_uncompressed_packet(
        &mut self,
        packet: RawPacket,
    ) -> Result<(), PacketStreamError> {
        let packet_length = packet.size();
        if packet_length > MAXIMUM_PACKET_LENGTH {
            return Err(PacketLengthParseError::PacketTooLarge.into());
        }

        let packet_length_bytes = VarInt::new(i32::try_from(packet_length)?).to_bytes();
        self.stream.write_all(&packet_length_bytes).await?;
        self.stream.write_all(packet.bytes()).await?;
        self.stream.flush().await?;

        Ok(())
    }

    async fn write_compressed_packet_format(
        &mut self,
        packet: RawPacket,
        compression_settings: &CompressionSettings,
    ) -> Result<(), PacketStreamError> {
        let uncompressed_payload = packet.bytes();
        let uncompressed_len = uncompressed_payload.len();

        let (data_length_bytes, final_payload) =
            if uncompressed_len >= compression_settings.threshold {
                // Compress the packet
                let data_length = VarInt::new(i32::try_from(uncompressed_len)?).to_bytes();
                let compressed_payload =
                    compress_data(uncompressed_payload, compression_settings.level)?;
                (data_length, compressed_payload)
            } else {
                // Don't compress, send with data length 0
                let data_length = VarInt::new(0).to_bytes();
                (data_length, uncompressed_payload.to_vec())
            };

        let packet_length = data_length_bytes.len() + final_payload.len();
        let packet_length_bytes = VarInt::new(i32::try_from(packet_length)?).to_bytes();

        self.stream.write_all(&packet_length_bytes).await?;
        self.stream.write_all(&data_length_bytes).await?;
        self.stream.write_all(&final_payload).await?;
        self.stream.flush().await?;

        Ok(())
    }

    async fn read_and_validate_packet_length(&mut self) -> Result<usize, PacketStreamError> {
        let packet_length = usize::try_from(self.read_var_int().await?)?;

        if packet_length > MAXIMUM_PACKET_LENGTH {
            return Err(PacketStreamError::PacketTooLarge {
                size: packet_length,
                max: MAXIMUM_PACKET_LENGTH,
            });
        }

        if packet_length == 0 {
            return Err(PacketStreamError::ZeroLengthPacket);
        }

        Ok(packet_length)
    }

    async fn read_var_int(&mut self) -> Result<i32, PacketStreamError> {
        let mut var_int_buf = Vec::with_capacity(5);

        for _ in 0..5 {
            let mut byte = [0u8; 1];
            self.stream.read_exact(&mut byte).await?;
            var_int_buf.push(byte[0]);

            // Try to parse, but continue if more data is needed.
            match get_packet_length(&var_int_buf) {
                Ok(length) => return Ok(i32::try_from(length)?),
                Err(PacketLengthParseError::BinaryReader(BinaryReaderError::UnexpectedEof)) => {}
                Err(e) => return Err(e.into()),
            }
        }

        // If we loop 5 times and still need more data, the VarInt is too long.
        Err(PacketLengthParseError::VarIntTooLong.into())
    }
}

#[derive(Error, Debug)]
pub enum PacketStreamError {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    LengthParse(#[from] PacketLengthParseError),
    #[error("received a packet with length 0")]
    ZeroLengthPacket,
    #[error("received packet with length {size} which exceeds the maximum of {max}")]
    PacketTooLarge { size: usize, max: usize },
    #[error("packet is missing a packet ID")]
    MissingPacketId,
    #[error(transparent)]
    BinaryReader(#[from] BinaryReaderError),
    #[error("failed to decompress packet data: {0}")]
    DecompressionIo(#[source] io::Error),
    #[error("decompressed size mismatch: expected {expected}, got {actual}")]
    DecompressionSizeMismatch { expected: usize, actual: usize },
    #[error(transparent)]
    TryFromInt(#[from] TryFromIntError),
}

fn compress_data(data: &[u8], compression_level: Compression) -> io::Result<Vec<u8>> {
    let mut encoder = ZlibEncoder::new(Vec::new(), compression_level);
    encoder.write_all(data)?;
    encoder.finish()
}

fn decompress_data(
    compressed_data: &[u8],
    uncompressed_size: usize,
) -> Result<Vec<u8>, PacketStreamError> {
    let mut decoder = ZlibDecoder::new(compressed_data);
    let mut decompressed_data = Vec::with_capacity(uncompressed_size);

    decoder
        .read_to_end(&mut decompressed_data)
        .map_err(PacketStreamError::DecompressionIo)?;

    if decompressed_data.len() != uncompressed_size {
        return Err(PacketStreamError::DecompressionSizeMismatch {
            expected: uncompressed_size,
            actual: decompressed_data.len(),
        });
    }

    Ok(decompressed_data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    // Uncompressed Read Tests

    #[tokio::test]
    async fn test_read_simple_packet() {
        // Given
        let reader = tokio_test::io::Builder::new().read(&[1, 42]).build();
        let mut packet_stream = PacketStream::new(reader);

        // When
        let packet = packet_stream.read_packet().await.unwrap();

        // Then
        assert_eq!(packet.packet_id(), Some(42));
        assert_eq!(packet.size(), 1);
    }

    #[tokio::test]
    async fn test_read_zero_length_packet() {
        // Given
        let reader = tokio_test::io::Builder::new().read(&[0]).build();
        let mut packet_stream = PacketStream::new(reader);

        // When
        let result = packet_stream.read_packet().await;

        // Then
        assert!(matches!(result, Err(PacketStreamError::ZeroLengthPacket)));
    }

    #[tokio::test]
    async fn test_read_eof() {
        // Given
        let reader = tokio_test::io::Builder::new().read(&[2, 10]).build(); // Length 2, but only 1 byte follows
        let mut packet_stream = PacketStream::new(reader);

        // When
        let result = packet_stream.read_packet().await;

        // Then
        assert!(matches!(result, Err(PacketStreamError::Io(_))));
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
        assert_eq!(packet.packet_id(), Some(42));
        assert_eq!(packet.data(), &[84]);
    }

    #[tokio::test]
    async fn test_two_packets() {
        // Given
        let reader = tokio_test::io::Builder::new()
            .read(&[1, 42])
            .read(&[2, 80, 84])
            .build();
        let mut packet_stream = PacketStream::new(reader);

        // When
        let packet_1 = packet_stream.read_packet().await.unwrap();
        let packet_2 = packet_stream.read_packet().await.unwrap();

        // Then
        assert_eq!(packet_1.packet_id(), Some(42));
        assert_eq!(packet_2.packet_id(), Some(80));
        assert_eq!(packet_2.data(), &[84]);
    }

    // Uncompressed Write Tests

    #[tokio::test]
    async fn test_write_simple_packet() {
        // Given
        let packet = RawPacket::new(vec![42, 84]).unwrap();
        let expected_bytes = vec![2, 42, 84]; // Length, ID, Data
        let stream = tokio_test::io::Builder::new()
            .write(&expected_bytes)
            .build();
        let mut packet_stream = PacketStream::new(stream);

        // When / Then
        packet_stream.write_packet(packet).await.unwrap();
    }
}
