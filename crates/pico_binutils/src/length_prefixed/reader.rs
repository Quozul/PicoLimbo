use crate::binary_reader::ReadBytes;
use crate::prelude::{BinaryReader, BinaryReaderError, Prefixed};

pub trait ReadLengthPrefix: Sized + ReadBytes {
    fn read_usize(reader: &mut BinaryReader) -> Result<usize, BinaryReaderError>;
}

impl<L> ReadBytes for Prefixed<L, String>
where
    L: ReadLengthPrefix,
{
    #[inline]
    fn read(reader: &mut BinaryReader) -> Result<Self, BinaryReaderError> {
        let length = L::read_usize(reader)?;
        let mut string_bytes = Vec::with_capacity(length);
        for _ in 0..length {
            string_bytes.push(reader.read()?);
        }
        Ok(Prefixed::new(String::from_utf8(string_bytes)?))
    }
}

impl<L, T> ReadBytes for Prefixed<L, Vec<T>>
where
    L: ReadLengthPrefix,
    T: ReadBytes,
{
    #[inline]
    fn read(reader: &mut BinaryReader) -> Result<Self, BinaryReaderError> {
        let length = L::read_usize(reader)?;
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
    fn read_usize(reader: &mut BinaryReader) -> Result<usize, BinaryReaderError> {
        let len = reader.read()?;
        from_i32(len)
    }
}

impl ReadLengthPrefix for i16 {
    fn read_usize(reader: &mut BinaryReader) -> Result<usize, BinaryReaderError> {
        let len: i16 = reader.read()?;
        len.try_into().map_err(|_| {
            BinaryReaderError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid length: negative or too large for usize",
            ))
        })
    }
}
