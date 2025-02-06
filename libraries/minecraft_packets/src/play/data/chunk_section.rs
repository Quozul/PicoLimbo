use crate::play::data::palette_container::{PaletteContainer, PaletteContainerError};
use data_types::prelude::{EncodePacketField, VarInt};
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
    pub fn void() -> Self {
        Self {
            block_count: 0,
            block_states: PaletteContainer::void(),
            biomes: PaletteContainer::void(),
        }
    }
}

#[derive(Error, Debug)]
pub enum ChunkSectionError {
    #[error("invalid palette container error")]
    Infallible(#[from] std::convert::Infallible),
    #[error("error while decoding a palette container")]
    PaletteContainerError(#[from] PaletteContainerError),
}

impl EncodePacketField for ChunkSection {
    type Error = ChunkSectionError;

    fn encode(&self, bytes: &mut Vec<u8>) -> Result<(), Self::Error> {
        VarInt::new(self.block_count as i32).encode(bytes)?;
        self.block_states.encode(bytes)?;
        self.biomes.encode(bytes)?;
        Ok(())
    }
}
