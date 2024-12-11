use crate::prelude::SerializePacketData;
use thiserror::Error;

#[derive(Error, Debug)]
#[error("invalid vec no length error")]
pub enum VecEncodeError {
    EncodeError,
}

impl<T: SerializePacketData> SerializePacketData for Vec<T> {
    type Error = VecEncodeError;

    fn encode(&self, bytes: &mut Vec<u8>) -> Result<(), Self::Error> {
        for value in self {
            value
                .encode(bytes)
                .map_err(|_| VecEncodeError::EncodeError)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::data_types::var_int::VarInt;
    use crate::prelude::SerializePacketData;

    #[test]
    fn test_vec_encode() {
        let vec = vec![VarInt::new(1), VarInt::new(2)];
        let mut bytes = Vec::new();
        vec.encode(&mut bytes).unwrap();
        assert_eq!(bytes, vec![0x01, 0x02]);
    }

    #[test]
    fn test_vec_encode_empty() {
        let vec = Vec::<VarInt>::new();
        let mut bytes = Vec::new();
        vec.encode(&mut bytes).unwrap();
        assert!(bytes.is_empty());
    }
}
