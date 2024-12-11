use crate::deserialize_packet::DeserializePacketData;
use crate::prelude::{SerializePacketData, VarInt};
use std::fmt::Debug;
use thiserror::Error;

/// A wrapper around a Vec that adds the length as a VarInt before the Vec itself.
#[derive(Debug, Clone, Default)]
pub struct LengthPaddedVec<T>(pub Vec<T>);

#[derive(Error, Debug)]
pub enum LengthPaddedVecDecodeError<T: DeserializePacketData> {
    #[error("vec length is invalid")]
    InvalidVecLength,
    #[error("error while decoding a value from the vec; error={0}")]
    DecodeError(T::Error),
}

impl<T: DeserializePacketData + Debug> DeserializePacketData for LengthPaddedVec<T> {
    type Error = LengthPaddedVecDecodeError<T>;

    fn decode(bytes: &[u8], index: &mut usize) -> Result<Self, Self::Error> {
        let length = VarInt::decode(bytes, index)
            .map_err(|_| LengthPaddedVecDecodeError::InvalidVecLength)?
            .value();

        let mut vec = Vec::with_capacity(length as usize);

        for _ in 0..length {
            vec.push(T::decode(bytes, index).map_err(LengthPaddedVecDecodeError::DecodeError)?);
        }

        Ok(LengthPaddedVec(vec))
    }
}

#[derive(Error, Debug)]
#[error("invalid vec error")]
pub enum LengthPaddedVecEncodeError {
    EncodeError,
}

impl<T: SerializePacketData> SerializePacketData for LengthPaddedVec<T> {
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
    use crate::deserialize_packet::DeserializePacketData;
    use crate::prelude::SerializePacketData;

    #[test]
    fn test_vec_decode() {
        let bytes = vec![0x02, 0x01, 0x02];
        let mut index = 0;
        let vec = LengthPaddedVec::<VarInt>::decode(&bytes, &mut index).unwrap();
        assert_eq!(vec.0.len(), 2);
        assert_eq!(vec.0[0].value(), 1);
        assert_eq!(vec.0[1].value(), 2);
        assert_eq!(index, 3);
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
