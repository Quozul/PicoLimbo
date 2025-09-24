use crate::binary_reader::{BinaryReader, BinaryReaderError, ReadBytes};
use crate::binary_writer::WriteBytes;
use crate::prelude::{BinaryWriter, BinaryWriterError};
use std::io;
use std::io::Write;
use std::num::TryFromIntError;

pub const SEGMENT_BITS: u8 = 0x7F;
pub const CONTINUE_BIT: u8 = 0x80;

#[derive(Debug, Default, Clone, PartialEq, PartialOrd, Eq, Hash, Ord)]
pub struct VarInt(i32);

#[cfg(feature = "binary_reader")]
impl ReadBytes for VarInt {
    #[inline]
    fn read(reader: &mut BinaryReader) -> Result<Self, BinaryReaderError> {
        let mut num_read = 0;
        let mut result: u32 = 0;

        loop {
            let byte: u8 = reader.read()?;

            let value = (byte & SEGMENT_BITS) as u32;
            result |= value << (7 * num_read);

            num_read += 1;
            if num_read > 5 {
                return Err(BinaryReaderError::VarIntTooBig);
            }

            if byte & CONTINUE_BIT == 0 {
                break;
            }
        }

        Ok(VarInt(result as i32))
    }
}

#[cfg(feature = "binary_writer")]
impl WriteBytes for VarInt {
    fn write(&self, writer: &mut BinaryWriter) -> Result<(), BinaryWriterError> {
        self.write_to(&mut writer.0)?;
        Ok(())
    }
}

impl VarInt {
    pub fn new(value: i32) -> Self {
        Self(value)
    }

    pub fn inner(&self) -> i32 {
        self.0
    }

    pub fn to_bytes(&self) -> io::Result<Vec<u8>> {
        let mut bytes = Vec::with_capacity(5);
        self.write_to(&mut bytes)?;
        Ok(bytes)
    }

    fn write_to<W: Write>(&self, writer: &mut W) -> io::Result<usize> {
        let mut value = self.0 as u32;
        let mut bytes_written = 0;

        while value >= 0x80 {
            let byte_to_write = (value as u8) | CONTINUE_BIT;
            writer.write_all(&[byte_to_write])?;
            bytes_written += 1;
            value >>= 7;
        }

        writer.write_all(&[value as u8])?;
        bytes_written += 1;

        Ok(bytes_written)
    }
}

impl From<i32> for VarInt {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

impl From<&i32> for VarInt {
    fn from(value: &i32) -> Self {
        Self::from(*value)
    }
}

impl From<u32> for VarInt {
    fn from(value: u32) -> Self {
        Self(value as i32)
    }
}

impl From<&u32> for VarInt {
    fn from(value: &u32) -> Self {
        Self::from(*value)
    }
}

impl TryFrom<i64> for VarInt {
    type Error = TryFromIntError;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        Ok(Self::from(i32::try_from(value)?))
    }
}

impl TryFrom<usize> for VarInt {
    type Error = TryFromIntError;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        Ok(Self::from(i32::try_from(value)?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_cases() -> Vec<(Vec<u8>, i32)> {
        vec![
            (vec![0x00], 0),
            (vec![0x01], 1),
            (vec![0x02], 2),
            (vec![0x7f], 127),
            (vec![0x80, 0x01], 128),
            (vec![0xff, 0x01], 255),
            (vec![0xdd, 0xc7, 0x01], 25565),
            (vec![0xff, 0xff, 0x7f], 2097151),
            (vec![0xff, 0xff, 0xff, 0xff, 0x07], 2147483647),
            (vec![0xff, 0xff, 0xff, 0xff, 0x0f], -1),
            (vec![0x80, 0x80, 0x80, 0x80, 0x08], -2147483648),
        ]
    }

    fn get_read_test_cases() -> Vec<(Vec<u8>, i32)> {
        vec![
            (vec![0x01, 0x09], 1),
            (
                vec![0x09, 0x31, 0x32, 0x37, 0x2e, 0x30, 0x2e, 0x30, 0x2e, 0x31],
                9,
            ),
        ]
    }

    #[test]
    fn test_read_var_int() {
        for (bytes, expected) in get_test_cases() {
            let mut reader = BinaryReader::new(&bytes);
            let result: VarInt = reader.read().unwrap();
            assert_eq!(result.inner(), expected);
        }

        for (bytes, expected) in get_read_test_cases() {
            let mut reader = BinaryReader::new(&bytes);
            let result: VarInt = reader.read().unwrap();
            assert_eq!(result.inner(), expected);
        }
    }

    #[test]
    fn test_decode_var_int_insufficient_bytes() {
        let bytes = vec![];
        let mut reader = BinaryReader::new(&bytes);
        let result = reader.read::<VarInt>();
        assert!(result.is_err());
    }
}
