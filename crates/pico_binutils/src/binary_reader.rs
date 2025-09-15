use std::io::{Cursor, Read};
use std::string::FromUtf8Error;
use thiserror::Error;

pub trait ReadBytes: Sized {
    fn read(reader: &mut BinaryReader) -> Result<Self, BinaryReaderError>;
}

pub struct BinaryReader<'a>(pub(crate) Cursor<&'a [u8]>);

impl<'a> BinaryReader<'a> {
    pub fn new(raw: &'a [u8]) -> Self {
        Self(Cursor::new(raw))
    }

    pub fn read<T: ReadBytes>(&mut self) -> Result<T, BinaryReaderError> {
        T::read(self)
    }

    pub fn read_bytes(&mut self, buf: &mut [u8]) -> Result<usize, BinaryReaderError> {
        self.0.read(buf).map_err(BinaryReaderError::from)
    }

    pub fn remaining(&self) -> usize {
        let total_len = self.0.get_ref().len();
        let current_pos = self.0.position() as usize;
        total_len.saturating_sub(current_pos)
    }

    pub fn remaining_bytes(&mut self) -> Result<Vec<u8>, BinaryReaderError> {
        let mut buf = vec![0u8; self.remaining()];
        self.read_bytes(&mut buf)?;
        Ok(buf)
    }

    pub fn position(&self) -> u64 {
        self.0.position()
    }
}

#[derive(Debug, Error)]
pub enum BinaryReaderError {
    #[error("unexpected eof")]
    UnexpectedEof,
    #[error(transparent)]
    Io(std::io::Error),
    #[error(transparent)]
    InvalidUtf8(#[from] FromUtf8Error),
    #[cfg(feature = "var_int")]
    #[error("var int too big")]
    VarIntTooBig,
    #[error("custom error")]
    Custom,
}

impl From<std::io::Error> for BinaryReaderError {
    fn from(err: std::io::Error) -> Self {
        match err.kind() {
            std::io::ErrorKind::UnexpectedEof => BinaryReaderError::UnexpectedEof,
            _ => BinaryReaderError::Io(err),
        }
    }
}

macro_rules! impl_read_int {
    ($($t:ty),*) => {
        $(
            impl ReadBytes for $t {
                #[inline]
                fn read(reader: &mut BinaryReader) -> Result<Self, BinaryReaderError> {
                    let size = std::mem::size_of::<$t>();
                    let mut bytes = vec![0u8; size];
                    reader.0.read_exact(&mut bytes)?;
                    let arr: [u8; std::mem::size_of::<$t>()] = bytes.try_into()
                        .map_err(|_| BinaryReaderError::UnexpectedEof)?;
                    let value = <$t>::from_be_bytes(arr);
                    Ok(value)
                }
            }
        )*
    }
}

impl_read_int!(u8, i8, u16, i16, u64, u32, i32, i64, usize, f32, f64);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::UShortPrefixed;

    #[test]
    fn test_read_i8() {
        let data = [0x7F];
        let mut reader = BinaryReader::new(&data);
        assert_eq!(reader.read::<i8>().unwrap(), 127);
    }

    #[test]
    fn test_read_two_i8() {
        let data = [0x7F, 0xFF];
        let mut reader = BinaryReader::new(&data);
        assert_eq!(reader.read::<i8>().unwrap(), 127);
        assert_eq!(reader.read::<i8>().unwrap(), -1);
    }

    #[test]
    fn test_read_i16() {
        let data = [0x7F, 0xFF];
        let mut reader = BinaryReader::new(&data);
        assert_eq!(reader.read::<i16>().unwrap(), 32767);
    }

    #[test]
    fn test_read_u16() {
        let data = [0x0F, 0xFF];
        let mut reader = BinaryReader::new(&data);
        assert_eq!(reader.read::<u16>().unwrap(), 4095);
    }

    #[test]
    fn test_read_i32() {
        let data = [0x7F, 0xFF, 0xFF, 0xFF];
        let mut reader = BinaryReader::new(&data);
        assert_eq!(reader.read::<i32>().unwrap(), 2147483647);
    }

    #[test]
    fn test_read_f32() {
        let data = [0x3F, 0x80, 0x00, 0x00];
        let mut reader = BinaryReader::new(&data);
        assert_eq!(reader.read::<f32>().unwrap(), 1.0);
    }

    #[test]
    fn test_read_string() {
        let data = [0, 5, 72, 69, 76, 76, 79];
        let mut reader = BinaryReader::new(&data);
        let parsed = reader.read::<UShortPrefixed<String>>().unwrap();

        assert_eq!(parsed.into_inner(), "HELLO");
    }
}
