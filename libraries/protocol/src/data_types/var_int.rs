use crate::prelude::EncodePacketField;
use crate::traits::decode_packet_field::DecodePacketField;
use thiserror::Error;
use tokio::io::{AsyncRead, AsyncReadExt};

pub const SEGMENT_BITS: u8 = 0x7F;
pub const CONTINUE_BIT: u8 = 0x80;

#[derive(Error, Debug)]
pub enum VarIntParseError {
    #[error("invalid var int")]
    VarIntTooLarge,
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Default, Clone)]
pub struct VarInt(i32);

impl VarInt {
    pub fn new(value: i32) -> Self {
        Self(value)
    }

    pub fn value(&self) -> i32 {
        self.0
    }
}

#[async_trait::async_trait]
impl DecodePacketField for VarInt {
    type Error = VarIntParseError;

    async fn decode<T>(reader: &mut T) -> Result<Self, Self::Error>
    where
        T: AsyncRead + Unpin + Send,
    {
        let mut value = 0;
        let mut position = 0;

        while position < 32 {
            let mut buf = [0u8; 1];
            reader.read_exact(&mut buf).await?;
            let current_byte = buf[0];

            value |= ((current_byte & SEGMENT_BITS) as i32) << position;

            if (current_byte & CONTINUE_BIT) == 0 {
                break;
            }

            position += 7;
        }

        if position >= 32 {
            return Err(VarIntParseError::VarIntTooLarge);
        }

        Ok(Self(value))
    }
}

impl EncodePacketField for VarInt {
    type Error = std::convert::Infallible;

    fn encode(&self, bytes: &mut Vec<u8>) -> Result<(), Self::Error> {
        let mut value = self.value();

        loop {
            if (value & !(SEGMENT_BITS as i32)) == 0 {
                bytes.push(value as u8);
                break;
            }

            bytes.push(((value & SEGMENT_BITS as i32) as u8) | CONTINUE_BIT);
            value = (value as u32 >> 7) as i32;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::{DecodePacketField, EncodePacketField};

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

    #[tokio::test]
    async fn test_read_var_int() {
        let test_cases = get_test_cases();

        for (bytes, expected) in test_cases {
            let mut reader = tokio_test::io::Builder::new().read(&bytes).build();

            let result = VarInt::decode(&mut reader).await;
            assert_eq!(result.unwrap().value(), expected);
        }
    }

    #[test]
    fn test_encode_var_int() {
        let test_cases = get_test_cases();

        for (expected_bytes, value) in test_cases {
            let mut bytes = Vec::new();
            VarInt::new(value).encode(&mut bytes).unwrap();
            assert_eq!(bytes, expected_bytes);
        }
    }
}
