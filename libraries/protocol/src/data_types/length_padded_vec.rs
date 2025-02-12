use crate::prelude::{EncodePacketField, VarInt, VarIntParseError};
use crate::traits::decode_packet_field::DecodePacketField;
use std::fmt::Debug;
use thiserror::Error;
use tokio::io::AsyncRead;

/// A wrapper around a Vec that adds the length as a VarInt before the Vec itself.
#[derive(Debug, Clone, Default)]
pub struct LengthPaddedVec<T>(pub Vec<T>);

#[derive(Error, Debug)]
pub enum LengthPaddedVecDecodeError<T: DecodePacketField> {
    #[error(transparent)]
    InvalidVecLength(#[from] VarIntParseError),
    #[error("error while decoding a value from the vec; error={0}")]
    DecodeError(T::Error),
}

#[async_trait::async_trait]
impl<T: DecodePacketField + Debug + Send> DecodePacketField for LengthPaddedVec<T> {
    type Error = LengthPaddedVecDecodeError<T>;

    async fn decode<R>(reader: &mut R) -> Result<Self, Self::Error>
    where
        R: AsyncRead + Unpin + Send,
    {
        let length = VarInt::decode(reader).await?.value();

        let mut vec = Vec::with_capacity(length as usize);

        for _ in 0..length {
            vec.push(
                T::decode(reader)
                    .await
                    .map_err(LengthPaddedVecDecodeError::DecodeError)?,
            );
        }

        Ok(LengthPaddedVec(vec))
    }
}

#[derive(Error, Debug)]
#[error("invalid vec error")]
pub enum LengthPaddedVecEncodeError {
    EncodeError,
}

impl<T: EncodePacketField> EncodePacketField for LengthPaddedVec<T> {
    type Error = LengthPaddedVecEncodeError;

    fn encode(&self, bytes: &mut Vec<u8>) -> Result<(), Self::Error> {
        VarInt::new(self.0.len() as i32)
            .encode(bytes)
            .map_err(|_| LengthPaddedVecEncodeError::EncodeError)?;

        for value in &self.0 {
            value
                .encode(bytes)
                .map_err(|_| LengthPaddedVecEncodeError::EncodeError)?;
        }
        Ok(())
    }
}

impl<T> From<Vec<T>> for LengthPaddedVec<T> {
    fn from(vec: Vec<T>) -> Self {
        LengthPaddedVec(vec)
    }
}

#[cfg(test)]
mod tests {
    use crate::data_types::length_padded_vec::LengthPaddedVec;
    use crate::data_types::var_int::VarInt;
    use crate::prelude::EncodePacketField;
    use crate::traits::decode_packet_field::DecodePacketField;

    #[tokio::test]
    async fn test_vec_decode() {
        // Given
        let bytes = vec![0x02, 0x01, 0x02];
        let mut reader = tokio_test::io::Builder::new().read(&bytes).build();

        // When
        let vec = LengthPaddedVec::<VarInt>::decode(&mut reader)
            .await
            .unwrap();

        // Then
        assert_eq!(vec.0.len(), 2);
        assert_eq!(vec.0[0].value(), 1);
        assert_eq!(vec.0[1].value(), 2);
    }

    #[test]
    fn test_vec_encode() {
        let vec = LengthPaddedVec(vec![VarInt::new(1), VarInt::new(2)]);
        let mut bytes = Vec::new();
        vec.encode(&mut bytes).unwrap();
        assert_eq!(bytes, vec![0x02, 0x01, 0x02]);
    }

    #[test]
    fn test_vec_encode_empty() {
        let vec = LengthPaddedVec(Vec::<VarInt>::new());
        let mut bytes = Vec::new();
        vec.encode(&mut bytes).unwrap();
        assert_eq!(bytes, vec![0x00]);
    }
}
