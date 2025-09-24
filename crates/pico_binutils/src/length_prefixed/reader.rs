use crate::binary_reader::ReadBytes;
use crate::prelude::{BinaryReader, BinaryReaderError, Prefixed};
use std::io::Read;
use tracing::warn;

pub trait ReadLengthPrefix: Sized + ReadBytes {
    fn read_to_usize(reader: &mut BinaryReader) -> Result<usize, BinaryReaderError>;
}

impl<L> ReadBytes for Prefixed<L, String>
where
    L: ReadLengthPrefix,
{
    #[inline]
    fn read(reader: &mut BinaryReader) -> Result<Self, BinaryReaderError> {
        let length = L::read_to_usize(reader)?;
        let mut bytes = vec![0u8; length];
        reader.0.read_exact(&mut bytes)?;
        Ok(Prefixed::new(String::from_utf8(bytes).unwrap_or_else(
            |_| {
                warn!(
                    "Invalid string of length {} ended at index {}",
                    length,
                    reader.position()
                );
                create_repeated_string(length, '�')
            },
        )))
    }
}

fn create_repeated_string(length: usize, ch: char) -> String {
    std::iter::repeat_n(ch, length).collect()
}

impl<L, T> ReadBytes for Prefixed<L, Vec<T>>
where
    L: ReadLengthPrefix,
    T: ReadBytes,
{
    #[inline]
    fn read(reader: &mut BinaryReader) -> Result<Self, BinaryReaderError> {
        let length = L::read_to_usize(reader)?;
        let mut vec = Vec::with_capacity(length);
        for _ in 0..length {
            vec.push(reader.read()?);
        }
        Ok(Prefixed::new(vec))
    }
}

pub(crate) fn from_i32(len: i32) -> Result<usize, BinaryReaderError> {
    len.try_into().map_err(|_| {
        BinaryReaderError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Invalid length: negative or too large for usize",
        ))
    })
}

impl ReadLengthPrefix for i32 {
    fn read_to_usize(reader: &mut BinaryReader) -> Result<usize, BinaryReaderError> {
        let len = reader.read()?;
        from_i32(len)
    }
}

impl ReadLengthPrefix for u16 {
    fn read_to_usize(reader: &mut BinaryReader) -> Result<usize, BinaryReaderError> {
        Ok(reader.read::<u16>()?.into())
    }
}
