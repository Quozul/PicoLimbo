use crate::binary_reader::{BinaryReader, BinaryReaderError, ReadBytes};
use crate::binary_writer::WriteBytes;
use crate::prelude::{BinaryWriter, BinaryWriterError};

pub const SEGMENT_BITS: u8 = 0x7F;
pub const CONTINUE_BIT: u8 = 0x80;

#[derive(Debug, Default, Clone, PartialEq, PartialOrd)]
pub struct VarInt(i32);

impl VarInt {
    pub fn new(value: i32) -> Self {
        VarInt(value)
    }

    pub fn inner(&self) -> i32 {
        self.0
    }
}

impl From<i32> for VarInt {
    fn from(value: i32) -> Self {
        VarInt(value)
    }
}

impl From<&i32> for VarInt {
    fn from(value: &i32) -> Self {
        VarInt::from(*value)
    }
}

impl From<i64> for VarInt {
    fn from(value: i64) -> Self {
        Self::new(value as i32)
    }
}

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
        let mut value = self.0 as u32;
        loop {
            let mut temp = (value & (SEGMENT_BITS as u32)) as u8;
            value >>= 7;
            if value != 0 {
                temp |= CONTINUE_BIT;
            }
            writer.write(&temp)?;
            if value == 0 {
                break;
            }
        }
        Ok(())
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
