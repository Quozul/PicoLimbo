use crate::data_types::var_int::VarInt;
use crate::prelude::{DecodePacketField, EncodePacketField, VarIntParseError};
use thiserror::Error;
use tokio::io::{AsyncRead, AsyncReadExt};

#[derive(Error, Debug)]
pub enum StringDecodingError {
    #[error("string too large")]
    StringTooLarge,
    #[error("invalid utf-8 string")]
    InvalidUtf8String(#[from] std::str::Utf8Error),
    #[error(transparent)]
    VarIntParseError(#[from] VarIntParseError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

const MAX_STRING_SIZE: usize = 32767;

#[async_trait::async_trait]
impl DecodePacketField for String {
    type Error = StringDecodingError;

    async fn decode<T>(reader: &mut T) -> Result<Self, Self::Error>
    where
        T: AsyncRead + Unpin + Send,
    {
        let varint = VarInt::decode(reader).await?;
        let length = varint.value() as usize;
        if length > MAX_STRING_SIZE {
            return Err(StringDecodingError::StringTooLarge);
        }

        // For an empty string, do not consume any further bytes.
        if length == 0 {
            return Ok(String::new());
        }

        // Now read the rest of the string bytes.
        let mut string_bytes = vec![0u8; length];
        reader.read_exact(&mut string_bytes).await?;

        let s = std::str::from_utf8(&string_bytes)?;
        Ok(s.to_string())
    }
}

impl EncodePacketField for String {
    type Error = std::convert::Infallible;

    fn encode(&self, bytes: &mut Vec<u8>) -> Result<(), Self::Error> {
        VarInt::new(self.len() as i32).encode(bytes)?;
        bytes.extend_from_slice(self.as_bytes());
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::prelude::{DecodePacketField, EncodePacketField};

    #[test]
    fn test_encode_string() {
        let mut bytes = Vec::new();
        "hello".to_string().encode(&mut bytes).unwrap();
        assert_eq!(bytes, vec![5, 104, 101, 108, 108, 111]);
    }

    #[tokio::test]
    async fn test_decode_string() {
        // Given
        let bytes = vec![5, 104, 101, 108, 108, 111];
        let mut reader = tokio_test::io::Builder::new().read(&bytes).build();
        let expected_string = "hello".to_string();

        // When
        let string = String::decode(&mut reader).await.unwrap();

        // Then
        assert_eq!(string, expected_string);
    }
}
