use crate::play::data::palette_container::{PaletteContainer, PaletteContainerError};
use minecraft_protocol::prelude::*;
use minecraft_protocol::protocol_version::ProtocolVersion;
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct ChunkSection {
    /// Number of non-air blocks present in the chunk section.
    pub block_count: i16,
    /// Consists of 4096 entries, representing all the blocks in the chunk section.
    pub block_states: PaletteContainer,
    /// Consists of 64 entries, representing 4×4×4 biome regions in the chunk section.
    pub biomes: PaletteContainer,
}

impl ChunkSection {
    pub fn void(protocol_version: ProtocolVersion) -> Self {
        Self {
            block_count: 0,
            block_states: PaletteContainer::blocks_void(),
            biomes: PaletteContainer::biomes_void(protocol_version),
        }
    }
}

#[derive(Error, Debug)]
pub enum ChunkSectionError {
    #[error("error while decoding a palette")]
    EncodeError,
    #[error("invalid palette container error")]
    Infallible,
    #[error("error while decoding a palette container")]
    PaletteContainerError,
}

impl From<std::convert::Infallible> for ChunkSectionError {
    fn from(_: std::convert::Infallible) -> Self {
        ChunkSectionError::Infallible
    }
}

impl<T: DecodePacketField> From<LengthPaddedVecDecodeError<T>> for ChunkSectionError {
    fn from(_: LengthPaddedVecDecodeError<T>) -> Self {
        ChunkSectionError::EncodeError
    }
}

impl From<PaletteContainerError> for ChunkSectionError {
    fn from(_: PaletteContainerError) -> Self {
        ChunkSectionError::PaletteContainerError
    }
}

impl EncodePacketField for ChunkSection {
    type Error = ChunkSectionError;

    fn encode(&self, bytes: &mut Vec<u8>, protocol_version: u32) -> Result<(), Self::Error> {
        self.block_count.encode(bytes, protocol_version)?;
        self.block_states.encode(bytes, protocol_version)?;
        self.biomes.encode(bytes, protocol_version)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_section_before_1_21_5() {
        let chunk_section = ChunkSection::void(ProtocolVersion::V1_21_4);

        let mut buffer = Vec::new();
        chunk_section.encode(&mut buffer, 769).unwrap();

        assert_eq!(
            buffer,
            vec![
                /* Block count */
                0x00, 0x00,
                /* Block states */
                /* Bits Per Entry */
                0x00, /* Palette */
                /* Value */
                0x00, /* Data Array Length */
                0x00, /* Biomes */
                /* Bits Per Entry */
                0x00, /* Value */
                0x01, /* Data Array Length */
                0x00
            ]
        );
        assert_eq!(buffer.len(), 8);
    }

    #[test]
    fn test_chunk_section_after_1_21_5() {
        let chunk_section = ChunkSection::void(ProtocolVersion::V1_21_5);

        let mut buffer = Vec::new();
        chunk_section.encode(&mut buffer, 770).unwrap();

        assert_eq!(
            buffer,
            vec![
                /* Block count */
                0x00, 0x00,
                /* Block states */
                /* Bits Per Entry */
                0x00, /* Palette */
                /* Value */
                0x00, /* Data Array Length */
                0x00, /* Biomes */
                /* Bits Per Entry */
                0x00, /* Value */
                0x00, /* Data Array Length */
                0x00
            ]
        );
        assert_eq!(buffer.len(), 8);
    }
}
