use protocol_version::protocol_version::ProtocolVersion;
use std::fmt::Display;
use std::io;
use std::path::StripPrefixError;

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum Error {
    Io = 0,
    Identifier = 1,
    Json = 2,
    Nbt = 3,
    StripPrefix = 4,
    IncompatibleVersion = 5,
    UnknownRegistryEntry = 6,
    UnknownTagEntry = 7,
    UnknownRegistry = 8,
    RegistryEntryNotOfExpectedType = 9,
    DataPathNotFound = 10,
    UnsupportedBiome = 11,
    BiomeIdUnsupportedVersion = 12,
    DimensionCodecUnsupportedVersion = 13,
    RegistryCodecUnsupportedVersion = 14,
    DimensionInfoUnsupportedVersion = 15,
    RegistryDataUnsupportedVersion = 16,
    TaggedRegistriesUnsupportedVersion = 17,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "REG_{:03}", *self as u8)
    }
}

impl std::error::Error for Error {}

impl From<io::Error> for Error {
    fn from(_: io::Error) -> Self {
        Self::Io
    }
}

impl From<pico_identifier::prelude::IdentifierParseError> for Error {
    fn from(_: pico_identifier::prelude::IdentifierParseError) -> Self {
        Self::Identifier
    }
}

impl From<serde_json::error::Error> for Error {
    fn from(_: serde_json::error::Error) -> Self {
        Self::Json
    }
}

impl From<pico_nbt2::Error> for Error {
    fn from(_: pico_nbt2::Error) -> Self {
        Self::Nbt
    }
}

impl From<StripPrefixError> for Error {
    fn from(_: StripPrefixError) -> Self {
        Self::StripPrefix
    }
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
            Err(Self::IncompatibleVersion)
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;
