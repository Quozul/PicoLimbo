use crate::prelude::*;
use thiserror::Error;

#[derive(Error, Debug)]
#[error("invalid option error")]
pub enum DecodeOptionError {
    DecodeError,
    Infallible,
}

impl<T: DecodePacketField> DecodePacketField for Option<T> {
    type Error = DecodeOptionError;

    fn decode(bytes: &[u8], index: &mut usize) -> Result<Self, Self::Error> {
        let present = bool::decode(bytes, index).map_err(|_| DecodeOptionError::Infallible)?;
        if present {
            Ok(Some(
                T::decode(bytes, index).map_err(|_| DecodeOptionError::DecodeError)?,
            ))
        } else {
            Ok(None)
        }
    }
}

#[derive(Error, Debug)]
#[error("invalid option error")]
pub enum EncodeOptionError {
    EncodeError,
}

impl<T: EncodePacketField> EncodePacketField for Option<T> {
    type Error = EncodeOptionError;

    fn encode(&self, bytes: &mut Vec<u8>) -> Result<(), Self::Error> {
        if let Some(value) = self {
            value
                .encode(bytes)
                .map_err(|_| EncodeOptionError::EncodeError)?;
        }
        Ok(())
    }
}
