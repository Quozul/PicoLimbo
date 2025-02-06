use crate::prelude::*;
use crate::var_int::CONTINUE_BIT;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum StringDecodingError {
    #[error("invalid string size")]
    InvalidStringSize,
    #[error("string too large")]
    StringTooLarge,
    #[error("invalid string offset")]
    InvalidOffset,
    #[error("invalid utf-8 string")]
    InvalidUtf8String(#[from] std::str::Utf8Error),
}

const MAX_STRING_SIZE: usize = 32767;

impl DecodePacketField for String {
    type Error = StringDecodingError;

    fn decode(bytes: &[u8], index: &mut usize) -> Result<Self, Self::Error> {
        let length = VarInt::decode(bytes, index)
            .map_err(|_| StringDecodingError::InvalidStringSize)?
            .value() as usize;

        if length > MAX_STRING_SIZE {
            return Err(StringDecodingError::StringTooLarge);
        }

        while (bytes[*index] & CONTINUE_BIT) != 0 {
            *index += 1;
        }

        if *index + length > bytes.len() {
            return Err(StringDecodingError::InvalidOffset);
        }

        let result = std::str::from_utf8(&bytes[*index..*index + length])
            .map_err(StringDecodingError::InvalidUtf8String)?;

        *index += length;

        Ok(result.to_string())
    }
}

impl EncodePacketField for String {
    type Error = std::convert::Infallible;

    fn encode(&self, bytes: &mut Vec<u8>) -> Result<(), Self::Error> {
        let string_bytes = self.as_bytes();
        VarInt::new(string_bytes.len() as i32).encode(bytes)?;
        bytes.extend_from_slice(string_bytes);
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::prelude::EncodePacketField;

    #[test]
    fn test_encode_string() {
        let mut bytes = Vec::new();
        "hello".to_string().encode(&mut bytes).unwrap();
        assert_eq!(bytes, vec![5, 104, 101, 108, 108, 111]);
    }
}
