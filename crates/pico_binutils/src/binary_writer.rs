use std::io::Write;
use std::num::TryFromIntError;
use thiserror::Error;

pub trait WriteBytes {
    fn write(&self, writer: &mut BinaryWriter) -> Result<(), BinaryWriterError>;
}

#[derive(Debug, Default)]
pub struct BinaryWriter(pub(crate) Vec<u8>);

#[derive(Debug, Error)]
pub enum BinaryWriterError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    TryFromInt(#[from] TryFromIntError),
    #[error("This operation is unsupported")]
    UnsupportedOperation,
}

impl BinaryWriter {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn write<T: WriteBytes + ?Sized>(&mut self, value: &T) -> Result<(), BinaryWriterError> {
        value.write(self)
    }

    pub fn write_bytes(&mut self, bytes: &[u8]) -> Result<usize, BinaryWriterError> {
        self.0.write(bytes).map_err(BinaryWriterError::from)
    }

    pub fn into_inner(self) -> Vec<u8> {
        self.0
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn clear(&mut self) {
        self.0.clear();
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
    use crate::prelude::{IntPrefixed, UShortPrefixed};

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
        let input = UShortPrefixed::string("hello world");

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
