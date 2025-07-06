use std::string::FromUtf8Error;

pub trait ReadBytes: Sized {
    fn read(reader: &mut BinaryReader) -> Result<Self, BinaryReaderError>;
}

#[derive(Debug)]
pub enum BinaryReaderError {
    UnexpectedEof,
    InvalidUtf8(FromUtf8Error),
}

impl From<FromUtf8Error> for BinaryReaderError {
    fn from(err: FromUtf8Error) -> Self {
        BinaryReaderError::InvalidUtf8(err)
    }
}

macro_rules! impl_read_int {
    ($($t:ty),*) => {
        $(
            impl ReadBytes for $t {
                #[inline]
                fn read(reader: &mut BinaryReader) -> Result<Self, BinaryReaderError> {
                    let size = std::mem::size_of::<$t>();
                    if reader.index + size > reader.raw.len() {
                        return Err(BinaryReaderError::UnexpectedEof);
                    }

                    let bytes = &reader.raw[reader.index..reader.index + size];
                    let value = <$t>::from_be_bytes(bytes.try_into().unwrap());
                    reader.index += size;
                    Ok(value)
                }
            }
        )*
    }
}

impl_read_int!(u8, i8, u16, i16, u32, i32, i64, usize, f32, f64);

impl<T: ReadBytes> ReadBytes for Vec<T> {
    #[inline]
    fn read(reader: &mut BinaryReader) -> Result<Self, BinaryReaderError> {
        let length: i32 = reader.read()?;
        let mut vec = Vec::with_capacity(length as usize);

        for _ in 0..length {
            vec.push(reader.read()?);
        }

        Ok(vec)
    }
}

impl ReadBytes for String {
    #[inline]
    fn read(reader: &mut BinaryReader) -> Result<Self, BinaryReaderError> {
        let length: i16 = reader.read()?;
        let length = length as usize;

        if reader.index + length > reader.raw.len() {
            return Err(BinaryReaderError::UnexpectedEof);
        }

        let bytes = &reader.raw[reader.index..reader.index + length];
        reader.index += length;

        Ok(String::from_utf8(bytes.to_vec())?)
    }
}

pub struct BinaryReader<'a> {
    raw: &'a [u8],
    index: usize,
}

impl<'a> BinaryReader<'a> {
    pub fn new(raw: &'a [u8]) -> Self {
        Self { raw, index: 0 }
    }

    pub fn read<T: ReadBytes>(&mut self) -> Result<T, BinaryReaderError> {
        T::read(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_i8() {
        let data = [0x7F];
        let mut reader = BinaryReader::new(&data);
        assert_eq!(reader.read::<i8>().unwrap(), 127);
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
        let parsed = reader.read::<String>().unwrap();

        assert_eq!(parsed, "HELLO");
    }
}
