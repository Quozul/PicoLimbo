use crate::prelude::{DecodePacketField, EncodePacketField};
use std::fmt::Debug;
use thiserror::Error;

#[derive(Error, Debug)]
#[error("invalid vec no length error")]
pub enum VecEncodeError {
    EncodeError,
}

impl<T: EncodePacketField> EncodePacketField for Vec<T> {
    type Error = VecEncodeError;

    fn encode(&self, bytes: &mut Vec<u8>, protocol_version: i32) -> Result<(), Self::Error> {
        for value in self {
            value
                .encode(bytes, protocol_version)
                .map_err(|_| VecEncodeError::EncodeError)?;
        }
        Ok(())
    }
}

#[derive(Error, Debug)]
pub enum VecDecodeError<T: DecodePacketField> {
    #[error("vec length is invalid")]
    InvalidVecLength,
    #[error("error while decoding a value from the vec; error={0}")]
    DecodeError(T::Error),
}

/// Decoding a vec u8 implies reading elements until the buffer is exhausted.
impl DecodePacketField for Vec<u8> {
    type Error = VecDecodeError<u8>;

    fn decode(bytes: &[u8], index: &mut usize) -> Result<Self, Self::Error> {
        let vec = bytes.get(*index..).unwrap_or_default();
        let length = vec.len();

        *index += length;

        Ok(vec.to_vec())
    }
}

#[cfg(test)]
mod tests {
    use crate::data_types::var_int::VarInt;
    use crate::prelude::EncodePacketField;

    #[test]
    fn test_vec_encode() {
        let vec = vec![VarInt::new(1), VarInt::new(2)];
        let mut bytes = Vec::new();
        vec.encode(&mut bytes, 0).unwrap();
        assert_eq!(bytes, vec![0x01, 0x02]);
    }

    #[test]
    fn test_vec_encode_empty() {
        let vec = Vec::<VarInt>::new();
        let mut bytes = Vec::new();
        vec.encode(&mut bytes, 0).unwrap();
        assert!(bytes.is_empty());
    }
}
