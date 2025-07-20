use std::io::Write;

pub trait WriteBytes {
    fn write(&self, writer: &mut BinaryWriter) -> Result<(), BinaryWriterError>;
}

#[derive(Debug, Default)]
pub struct BinaryWriter(pub(crate) Vec<u8>);

#[derive(Debug)]
pub enum BinaryWriterError {
    Io(std::io::Error),
}

impl From<std::io::Error> for BinaryWriterError {
    fn from(err: std::io::Error) -> Self {
        BinaryWriterError::Io(err)
    }
}

impl BinaryWriter {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn write<T: WriteBytes + ?Sized>(&mut self, value: &T) -> Result<(), BinaryWriterError> {
        value.write(self)
    }

    pub fn into_inner(self) -> Vec<u8> {
        self.0
    }
}

macro_rules! impl_write_int {
    ($($t:ty),*) => {
        $(
            impl WriteBytes for $t {
                #[inline]
                fn write(&self, writer: &mut BinaryWriter) -> Result<(), BinaryWriterError> {
                    writer.0.write_all(&self.to_be_bytes())?;
                    Ok(())
                }
            }
        )*
    }
}

impl_write_int!(
    u8, i8, u16, i16, u32, i32, i64, u64, i128, u128, usize, f32, f64
);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::{IntPrefixed, ShortPrefixed};

    #[test]
    fn test_unsigned_byte() {
        // Given
        let mut writer = BinaryWriter::default();

        // When
        writer.write(&0_u8).unwrap();

        // Then
        assert_eq!(vec![0], writer.into_inner());
    }

    #[test]
    fn test_string() {
        // Given
        let mut writer = BinaryWriter::default();
        let input = ShortPrefixed::string("hello world");

        // When
        writer.write(&input).unwrap();

        // Then
        assert_eq!(
            vec![
                0, 11, // String length
                104, 101, 108, 108, 111, 32, 119, 111, 114, 108, 100 // String content
            ],
            writer.into_inner(),
        );
    }

    #[test]
    fn test_vec() {
        // Given
        let mut writer = BinaryWriter::default();
        let input = vec![1_u8, 2, 3];

        // When
        writer.write(&IntPrefixed::new(input)).unwrap();

        // Then
        assert_eq!(
            vec![
                0, 0, 0, 3, // Vec length
                1, 2, 3 // Data
            ],
            writer.into_inner(),
        );
    }
}
