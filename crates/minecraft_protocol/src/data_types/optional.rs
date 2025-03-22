use crate::prelude::EncodePacketField;
use thiserror::Error;

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
