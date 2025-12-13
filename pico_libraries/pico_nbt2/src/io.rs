use crate::error::Result;
use flate2::Compression;
use flate2::read::{GzDecoder, ZlibDecoder};
use flate2::write::{GzEncoder, ZlibEncoder};
use std::io::{BufRead, BufReader, Read, Write};

/// NBT compression type.
#[derive(Debug, Clone, Copy)]
pub enum CompressionType {
    /// No compression.
    None,
    /// Gzip compression.
    Gzip,
    /// Zlib compression.
    Zlib,
}

/// Creates a decompressing reader based on the compression type.
///
/// # Errors
/// This function is infallible for all current compression types.
pub fn decode<'a, R>(reader: R) -> Result<Box<dyn Read + 'a>>
where
    R: Read + 'a,
{
    let mut buf = BufReader::new(reader);

    let header = buf.fill_buf()?;

    let compression = detect_compression(header);

    let reader: Box<dyn Read> = match compression {
        CompressionType::None => Box::new(buf),
        CompressionType::Gzip => Box::new(GzDecoder::new(buf)),
        CompressionType::Zlib => Box::new(ZlibDecoder::new(buf)),
    };

    Ok(reader)
}

fn detect_compression(header: &[u8]) -> CompressionType {
    // Need at least two bytes to disambiguate gzip from the rest.
    if header.first() == Some(&0x1F) && header.get(1) == Some(&0x8B) {
        CompressionType::Gzip
    } else if header.first() == Some(&0x78) {
        // 0x78 is the first byte of *every* valid zlib stream (CMF byte).
        // The second byte (FLG) can be a handful of values, but we don't need
        // to validate it for detection purposes.
        CompressionType::Zlib
    } else {
        CompressionType::None
    }
}

/// Creates a compressing writer based on the compression type.
///
/// # Errors
/// This function is infallible for all current compression types.
pub fn encode<'a, W>(writer: W, compression: CompressionType) -> Result<Box<dyn Write + 'a>>
where
    W: Write + 'a,
{
    match compression {
        CompressionType::None => Ok(Box::new(writer)),
        CompressionType::Gzip => Ok(Box::new(GzEncoder::new(writer, Compression::default()))),
        CompressionType::Zlib => Ok(Box::new(ZlibEncoder::new(writer, Compression::default()))),
    }
}
