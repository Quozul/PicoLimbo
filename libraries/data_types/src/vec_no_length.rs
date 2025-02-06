use crate::prelude::*;
use thiserror::Error;

#[derive(Error, Debug)]
#[error("invalid vec no length error")]
pub enum VecEncodeError {
    EncodeError,
}

impl<T: EncodePacketField> EncodePacketField for Vec<T> {
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
    use crate::prelude::EncodePacketField;
    use crate::prelude::*;

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
