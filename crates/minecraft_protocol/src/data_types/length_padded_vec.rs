use crate::prelude::{EncodePacketField, VarInt};
use crate::traits::decode_packet_field::DecodePacketField;
use std::fmt::Debug;
use thiserror::Error;

/// A wrapper around a Vec that adds the length as a VarInt before the Vec itself.
#[derive(Debug, Clone, Default)]
pub struct LengthPaddedVec<T>(Vec<T>);

impl<T> LengthPaddedVec<T> {
    pub fn new(data: Vec<T>) -> Self {
        Self(data)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl<T> IntoIterator for LengthPaddedVec<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a, T> IntoIterator for &'a LengthPaddedVec<T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut LengthPaddedVec<T> {
    type Item = &'a mut T;
    type IntoIter = std::slice::IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter_mut()
    }
}

#[derive(Error, Debug)]
pub enum LengthPaddedVecDecodeError<T: DecodePacketField> {
    #[error("vec length is invalid")]
    InvalidVecLength,
    #[error("error while decoding a value from the vec; error={0}")]
    DecodeError(T::Error),
}

impl<T: DecodePacketField + Debug> DecodePacketField for LengthPaddedVec<T> {
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

impl<T: EncodePacketField> EncodePacketField for LengthPaddedVec<T> {
    type Error = LengthPaddedVecEncodeError;

    fn encode(&self, bytes: &mut Vec<u8>, protocol_version: i32) -> Result<(), Self::Error> {
        VarInt::new(self.0.len() as i32)
            .encode(bytes, protocol_version)
            .map_err(|_| LengthPaddedVecEncodeError::EncodeError)?;

        for value in &self.0 {
            value
                .encode(bytes, protocol_version)
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
    use super::*;

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
    fn test_vec_decode_length_no_data() {
        let bytes = vec![0x02];
        let mut index = 0;
        let result = LengthPaddedVec::<VarInt>::decode(&bytes, &mut index);
        assert!(result.is_err());
        assert_eq!(index, 1);
    }

    #[test]
    fn test_vec_encode() {
        let vec = LengthPaddedVec(vec![VarInt::new(1), VarInt::new(2)]);
        let mut bytes = Vec::new();
        vec.encode(&mut bytes, 0).unwrap();
        assert_eq!(bytes, vec![0x02, 0x01, 0x02]);
    }

    #[test]
    fn test_vec_encode_empty() {
        let vec = LengthPaddedVec(Vec::<VarInt>::new());
        let mut bytes = Vec::new();
        vec.encode(&mut bytes, 0).unwrap();
        assert_eq!(bytes, vec![0x00]);
    }
}
