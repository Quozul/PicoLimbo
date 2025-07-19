use crate::prelude::{DecodePacketField, EncodePacketField};
use std::fmt::Debug;
use thiserror::Error;

#[derive(Error, Debug)]
#[error("invalid option error")]
pub enum EncodeOptionError {
    EncodeError,
}

impl<T: EncodePacketField> EncodePacketField for Option<T> {
    type Error = EncodeOptionError;

    fn encode(&self, bytes: &mut Vec<u8>, protocol_version: i32) -> Result<(), Self::Error> {
        if let Some(value) = self {
            value
                .encode(bytes, protocol_version)
                .map_err(|_| EncodeOptionError::EncodeError)?;
        }
        Ok(())
    }
}

#[derive(Error, Debug)]
pub enum DecodeOptionError<T: DecodePacketField> {
    #[error("invalid option error")]
    Eof,
    #[error("error while decoding option; error={0}")]
    Inner(T::Error),
}

impl<T: DecodePacketField + Debug> DecodePacketField for Option<T> {
    type Error = DecodeOptionError<T>;

    fn decode(bytes: &[u8], index: &mut usize) -> Result<Self, Self::Error> {
        let is_present = bool::decode(bytes, index).map_err(|_| DecodeOptionError::Eof)?;
        if is_present {
            let inner = T::decode(bytes, index).map_err(DecodeOptionError::Inner)?;
            Ok(Some(inner))
        } else {
            Ok(None)
        }
    }
}
