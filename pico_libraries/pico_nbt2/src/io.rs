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

/// NBT decoder.
pub enum Decoder<R: Read> {
    /// No compression.
    None(BufReader<R>),
    /// Gzip compression.
    Gzip(GzDecoder<BufReader<R>>),
    /// Zlib compression.
    Zlib(ZlibDecoder<BufReader<R>>),
}

impl<R: Read> Read for Decoder<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            Self::None(r) => r.read(buf),
            Self::Gzip(r) => r.read(buf),
            Self::Zlib(r) => r.read(buf),
        }
    }
}

/// Creates a decompressing reader based on the compression type.
///
/// # Errors
/// This function is infallible for all current compression types.
pub fn decode<R>(reader: R) -> Result<Decoder<R>>
where
    R: Read,
{
    let mut buf = BufReader::new(reader);

    let header = buf.fill_buf()?;

    let compression = detect_compression(header);

    let reader = match compression {
        CompressionType::None => Decoder::None(buf),
        CompressionType::Gzip => Decoder::Gzip(GzDecoder::new(buf)),
        CompressionType::Zlib => Decoder::Zlib(ZlibDecoder::new(buf)),
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

/// NBT encoder.
pub enum Encoder<W: Write> {
    /// No compression.
    None(W),
    /// Gzip compression.
    Gzip(GzEncoder<W>),
    /// Zlib compression.
    Zlib(ZlibEncoder<W>),
}

impl<W: Write> Write for Encoder<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            Self::None(w) => w.write(buf),
            Self::Gzip(w) => w.write(buf),
            Self::Zlib(w) => w.write(buf),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            Self::None(w) => w.flush(),
            Self::Gzip(w) => w.flush(),
            Self::Zlib(w) => w.flush(),
        }
    }
}

impl<W: Write> Encoder<W> {
    /// Finishes the encoding process and returns the underlying writer.
    ///
    /// For compressed encoders, this writes the compression footer.
    ///
    /// # Errors
    /// Returns an I/O error if finishing the compression fails.
    pub fn finish(self) -> Result<W> {
        match self {
            Self::None(w) => Ok(w),
            Self::Gzip(w) => Ok(w.finish()?),
            Self::Zlib(w) => Ok(w.finish()?),
        }
    }
}

/// Creates a compressing writer based on the compression type.
///
/// # Errors
/// This function is infallible for all current compression types.
pub fn encode<W>(writer: W, compression: CompressionType) -> Result<Encoder<W>>
where
    W: Write,
{
    match compression {
        CompressionType::None => Ok(Encoder::None(writer)),
        CompressionType::Gzip => Ok(Encoder::Gzip(GzEncoder::new(
            writer,
            Compression::default(),
        ))),
        CompressionType::Zlib => Ok(Encoder::Zlib(ZlibEncoder::new(
            writer,
            Compression::default(),
        ))),
    }
}
