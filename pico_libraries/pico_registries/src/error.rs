use crate::RegistryKeys;
use pico_identifier::Identifier;
use protocol_version::protocol_version::ProtocolVersion;
use std::io;
use std::path::StripPrefixError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("Identifier parse error: {0}")]
    Identifier(#[from] pico_identifier::prelude::IdentifierParseError),

    #[error("Json parse error: {0}")]
    Json(#[from] serde_json::error::Error),

    #[error("Nbt error: {0}")]
    Nbt(#[from] pico_nbt2::Error),

    #[error("Failed to strip prefix: {0}")]
    StripPrefix(#[from] StripPrefixError),

    #[error("This function supports versions {minimum} to {maximum}, was called with {current}")]
    IncompatibleVersion {
        current: ProtocolVersion,
        minimum: ProtocolVersion,
        maximum: ProtocolVersion,
    },

    #[error("Unknown registry entry: {0}")]
    UnknownRegistryEntry(Identifier),

    #[error("Unknown tag entry: {0}")]
    UnknownTagEntry(Identifier),

    #[error("Unknown registry: {0}")]
    UnknownRegistry(RegistryKeys),

    #[error("this registry entry is not of the expected type")]
    NotOfType,

    #[error("Custom error: {0}")]
    Message(String),
}

impl Error {
    /// # Errors
    /// Creates an Incompatible Version error if the current version is out of the range
    pub fn incompatible_version(
        current: ProtocolVersion,
        minimum: ProtocolVersion,
        maximum: ProtocolVersion,
    ) -> Result<()> {
        if current.between_inclusive(minimum, maximum) {
            Ok(())
        } else {
            Err(Self::IncompatibleVersion {
                current,
                minimum,
                maximum,
            })
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

impl serde::ser::Error for Error {
    fn custom<T: std::fmt::Display>(msg: T) -> Self {
        Self::Message(msg.to_string())
    }
}

impl serde::de::Error for Error {
    fn custom<T: std::fmt::Display>(msg: T) -> Self {
        Self::Message(msg.to_string())
    }
}
