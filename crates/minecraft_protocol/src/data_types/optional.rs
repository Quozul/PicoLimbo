use crate::prelude::{DecodePacket, EncodePacket};
use pico_binutils::prelude::{BinaryReader, BinaryReaderError, BinaryWriter, BinaryWriterError};
use protocol_version::protocol_version::ProtocolVersion;
use std::fmt::Debug;

/// A type used only to encode packets and skip a field.
pub enum Omitted<T> {
    None,
    Some(T),
}

impl<T: EncodePacket> EncodePacket for Omitted<T> {
    fn encode(
        &self,
        writer: &mut BinaryWriter,
        protocol_version: ProtocolVersion,
    ) -> Result<(), BinaryWriterError> {
        if let Self::Some(value) = self {
            value.encode(writer, protocol_version)?;
        }
        Ok(())
    }
}

/// Value prefixed by a boolean indicating if the value is present
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash, Default)]
pub enum Optional<T> {
    #[default]
    None,
    Some(T),
}

impl<T> Optional<T> {
    pub fn unwrap_or(self, default: T) -> T {
        match self {
            Self::None => default,
            Self::Some(x) => x,
        }
    }
}

impl<T: EncodePacket> EncodePacket for Optional<T> {
    fn encode(
        &self,
        writer: &mut BinaryWriter,
        protocol_version: ProtocolVersion,
    ) -> Result<(), BinaryWriterError> {
        match self {
            Self::None => {
                writer.write::<u8>(&0x00_u8)?;
            }
            Self::Some(value) => {
                writer.write::<u8>(&0x01_u8)?;
                value.encode(writer, protocol_version)?;
            }
        }
        Ok(())
    }
}

impl<T: DecodePacket> DecodePacket for Optional<T> {
    fn decode(
        reader: &mut BinaryReader,
        protocol_version: ProtocolVersion,
    ) -> Result<Self, BinaryReaderError> {
        let is_present = bool::decode(reader, protocol_version)?;
        if is_present {
            let inner = T::decode(reader, protocol_version)?;
            Ok(Self::Some(inner))
        } else {
            Ok(Self::None)
        }
    }
}
