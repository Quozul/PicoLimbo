use crate::binary_reader::{BinaryReader, BinaryReaderError, ReadBytes};
use crate::binary_writer::WriteBytes;
use crate::prelude::{BinaryWriter, BinaryWriterError};
use std::io;
use std::io::Write;
use std::num::TryFromIntError;

pub const SEGMENT_BITS: u8 = 0x7F;
pub const CONTINUE_BIT: u8 = 0x80;

#[derive(Debug, Default, Clone, PartialEq, PartialOrd, Eq, Hash, Ord)]
pub struct VarLong(i64);

#[cfg(feature = "binary_reader")]
impl ReadBytes for VarLong {
    #[inline]
    fn read(reader: &mut BinaryReader) -> Result<Self, BinaryReaderError> {
        let mut num_read = 0;
        let mut result: u64 = 0;

        loop {
            let byte: u8 = reader.read()?;

            if num_read >= 10 {
                return Err(BinaryReaderError::VarLongTooBig);
            }

            let value = (byte & SEGMENT_BITS) as u64;
            result |= value << (7 * num_read);

            num_read += 1;

            if byte & CONTINUE_BIT == 0 {
                break;
            }
        }

        Ok(VarLong(result as i64))
    }
}

#[cfg(feature = "binary_writer")]
impl WriteBytes for VarLong {
    fn write(&self, writer: &mut BinaryWriter) -> Result<(), BinaryWriterError> {
        self.write_to(&mut writer.0)?;
        Ok(())
    }
}

impl VarLong {
    pub fn new(value: i64) -> Self {
        Self(value)
    }

    pub fn inner(&self) -> i64 {
        self.0
    }

    pub fn to_bytes(&self) -> io::Result<Vec<u8>> {
        let mut bytes = Vec::with_capacity(10);
        self.write_to(&mut bytes)?;
        Ok(bytes)
    }

    fn write_to<W: Write>(&self, writer: &mut W) -> io::Result<usize> {
        // Cast to u64 first so that >>= 7 is an unsigned (logical) right shift,
        // matching Java's >>> operator and correctly handling negative i64 values.
        let mut value = self.0 as u64;
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

impl From<i64> for VarLong {
    fn from(value: i64) -> Self {
        Self(value)
    }
}

impl From<&i64> for VarLong {
    fn from(value: &i64) -> Self {
        Self::from(*value)
    }
}

impl From<u64> for VarLong {
    fn from(value: u64) -> Self {
        Self(value as i64)
    }
}

impl From<&u64> for VarLong {
    fn from(value: &u64) -> Self {
        Self::from(*value)
    }
}

impl From<i32> for VarLong {
    fn from(value: i32) -> Self {
        Self(value as i64)
    }
}

impl TryFrom<usize> for VarLong {
    type Error = TryFromIntError;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        Ok(Self::from(i64::try_from(value)?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test cases: (encoded bytes, decoded i64 value)
    /// Values verified against wiki.vg VarLong specification.
    fn get_test_cases() -> Vec<(Vec<u8>, i64)> {
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
            // i64::MAX  — 9 bytes
            (
                vec![0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x7f],
                i64::MAX,
            ),
            // -1        — 10 bytes (all bits set)
            (
                vec![0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x01],
                -1,
            ),
            // i32::MIN as i64
            (
                vec![0x80, 0x80, 0x80, 0x80, 0xf8, 0xff, 0xff, 0xff, 0xff, 0x01],
                i32::MIN as i64,
            ),
            // i64::MIN  — 10 bytes (only the sign bit set)
            (
                vec![0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x01],
                i64::MIN,
            ),
        ]
    }

    #[test]
    fn test_read_var_long() {
        for (bytes, expected) in get_test_cases() {
            let mut reader = BinaryReader::new(&bytes);
            let result: VarLong = reader.read().unwrap();
            assert_eq!(result.inner(), expected);
        }
    }

    #[test]
    fn test_write_var_long() {
        for (expected_bytes, value) in get_test_cases() {
            let var_long = VarLong::new(value);
            let bytes = var_long.to_bytes().unwrap();
            assert_eq!(bytes, expected_bytes, "mismatch for value {value}");
        }
    }

    #[test]
    fn test_roundtrip_var_long() {
        for (_, value) in get_test_cases() {
            let encoded = VarLong::new(value).to_bytes().unwrap();
            let mut reader = BinaryReader::new(&encoded);
            let decoded: VarLong = reader.read().unwrap();
            assert_eq!(decoded.inner(), value);
        }
    }

    #[test]
    fn test_decode_var_long_insufficient_bytes() {
        let bytes = vec![];
        let mut reader = BinaryReader::new(&bytes);
        let result = reader.read::<VarLong>();
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_var_long_too_big() {
        // 11 bytes all with the continue bit set — must be rejected
        let bytes = vec![0x80u8; 11];
        let mut reader = BinaryReader::new(&bytes);
        let result = reader.read::<VarLong>();
        assert!(result.is_err());
    }
}
