use crate::data_types::var_int::{CONTINUE_BIT, VarInt};
use crate::prelude::{DecodePacketField, EncodePacketField};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DecodeStringError {
    #[error("invalid string size")]
    InvalidStringSize,
    #[error("string too large")]
    StringTooLarge,
    #[error("invalid string offset")]
    InvalidOffset,
    #[error("invalid utf-8 string")]
    InvalidUtf8String(#[from] std::str::Utf8Error),
    #[error("not enough bytes")]
    NotEnoughBytes,
}

const MAX_STRING_SIZE: usize = 32767;

impl DecodePacketField for String {
    type Error = DecodeStringError;

    fn decode(bytes: &[u8], index: &mut usize) -> Result<Self, Self::Error> {
        let length = VarInt::decode(bytes, index)
            .map_err(|_| DecodeStringError::InvalidStringSize)?
            .value() as usize;

        if length > MAX_STRING_SIZE {
            return Err(DecodeStringError::StringTooLarge);
        }

        while (bytes[*index] & CONTINUE_BIT) != 0 {
            *index += 1;
        }

        if *index + length > bytes.len() {
            return Err(DecodeStringError::InvalidOffset);
        }

        let string_bytes = bytes
            .get(*index..*index + length)
            .ok_or(DecodeStringError::NotEnoughBytes)?;
        let result =
            std::str::from_utf8(string_bytes).map_err(DecodeStringError::InvalidUtf8String)?;

        *index += length;

        Ok(result.to_string())
    }
}

impl EncodePacketField for String {
    type Error = std::convert::Infallible;

    fn encode(&self, bytes: &mut Vec<u8>, protocol_version: u32) -> Result<(), Self::Error> {
        VarInt::new(self.len() as i32).encode(bytes, protocol_version)?;
        bytes.extend_from_slice(self.as_bytes());
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::prelude::EncodePacketField;

    #[test]
    fn test_encode_string() {
        let mut bytes = Vec::new();
        "hello".to_string().encode(&mut bytes, 0).unwrap();
        assert_eq!(bytes, vec![5, 104, 101, 108, 108, 111]);
    }
}
