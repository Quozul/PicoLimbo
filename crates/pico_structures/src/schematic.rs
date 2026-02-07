use crate::internal_block_entity::BlockEntity;
use crate::schematic_file::SchematicFile;
use blocks_report::{BlockStateLookup, InternalMapping, StateData};
use minecraft_protocol::prelude::Coordinates;
use pico_binutils::prelude::BinaryReaderError;
use std::path::Path;
use thiserror::Error;
use tracing::warn;

#[derive(Error, Debug)]
pub enum SchematicError {
    #[error("Error decompressing or reading file: {0}")]
    Io(#[from] std::io::Error),
    #[error("Error decoding NBT data: {0}")]
    Nbt(#[from] pico_nbt2::Error),
    #[error("Error reading binary block data: {0}")]
    BinaryRead(#[from] BinaryReaderError),
    #[error("Missing NBT tag: {0}")]
    MissingTag(String),
    #[error("NBT tag '{0}' has an incorrect type")]
    IncorrectTagType(String),
    #[error("Unsupported schematic version: {0}. Only version 2 is supported.")]
    UnsupportedVersion(i32),
    #[error("Air internal ID not found")]
    AirNotFound,
}

pub struct Schematic {
    /// Palette mapping: palette index -> StateData
    palette: Vec<StateData>,
    /// Block data: flat vector storing palette indices, indexed by `y * length * width + z * width + x`.
    block_data: Vec<i32>,
    dimensions: Coordinates,
    air_palette_index: i32,
    block_entities: Vec<BlockEntity>,
}

impl Schematic {
    /// Loads a `.schem` file from the given path for a specific Minecraft protocol version.
    pub fn load_schematic_file(
        path: &Path,
        internal_mapping: &InternalMapping,
    ) -> Result<Self, SchematicError> {
        let schematic_file = SchematicFile::from_path(path)?;
        let dimensions = schematic_file.get_dimensions();
        let (palette, air_palette_index) =
            Self::get_palette_and_air_index(&schematic_file, internal_mapping)?;
        let block_data = schematic_file.get_block_data().to_vec();
        // let block_entities = Self::parse_block_entities(&schematic_file);
        let block_entities = Vec::new(); // FIXME: Reimplement block entities

        Ok(Self {
            palette,
            block_data,
            dimensions,
            air_palette_index,
            block_entities,
        })
    }

    fn get_palette_and_air_index(
        schematic_file: &SchematicFile,
        internal_mapping: &InternalMapping,
    ) -> Result<(Vec<StateData>, i32), SchematicError> {
        let max_schematic_id = schematic_file.get_block_palette_max();
        let block_state_lookup = BlockStateLookup::new(internal_mapping);

        const AIR_IDENTIFIER: &str = "minecraft:air";
        let internal_air_id = *block_state_lookup
            .parse_state_string(AIR_IDENTIFIER)
            .map_err(|_| SchematicError::AirNotFound)?;

        // Initialize palette with air at index 0
        let mut palette: Vec<StateData> = vec![internal_air_id; max_schematic_id + 1];
        let palette_nbt = schematic_file.get_palette();

        for (block_name, schematic_palette_id) in palette_nbt {
            if let Ok(state_data) = block_state_lookup.parse_state_string(&block_name)
                && let Some(entry) =
                    palette.get_mut(usize::try_from(*schematic_palette_id).unwrap_or_default())
            {
                *entry = *state_data;
            } else {
                warn!(
                    "Schematic palette contains ID {} which is greater than PaletteMax of {}. Skipping.",
                    schematic_palette_id, max_schematic_id
                );
            }
        }

        Ok((palette, 0))
    }

    fn parse_block_entities(schematic_file: &SchematicFile) -> Vec<BlockEntity> {
        schematic_file
            .get_block_entities()
            .map(|block_entities| {
                block_entities
                    .iter()
                    .filter_map(BlockEntity::from_nbt)
                    .collect::<Vec<BlockEntity>>()
            })
            .unwrap_or_default()
    }

    /// Converts a 3D coordinate within the schematic to a 1D index for the `block_data` vector.
    /// The schematic format iterates Y, then Z, then X.
    #[inline]
    fn position_to_index(&self, position: Coordinates) -> usize {
        let width = self.dimensions.x() as usize;
        let length = self.dimensions.z() as usize;
        let x = position.x() as usize;
        let y = position.y() as usize;
        let z = position.z() as usize;

        (y * length * width) + (z * width) + x
    }

    fn is_out_of_bounds(&self, position: &Coordinates) -> bool {
        position.x() < 0
            || position.y() < 0
            || position.z() < 0
            || position.x() >= self.dimensions.x()
            || position.y() >= self.dimensions.y()
            || position.z() >= self.dimensions.z()
    }

    /// Gets the internal block state ID at the given relative coordinates within the schematic.
    pub fn get_block_state_id(&self, schematic_position: Coordinates) -> &StateData {
        if self.is_out_of_bounds(&schematic_position) {
            return &self.palette[self.air_palette_index as usize];
        }

        let index = self.position_to_index(schematic_position);
        let palette_index = self
            .block_data
            .get(index)
            .copied()
            .unwrap_or(self.air_palette_index);

        &self.palette[palette_index as usize]
    }

    pub fn get_dimensions(&self) -> Coordinates {
        self.dimensions
    }

    pub fn get_block_entities(&self) -> &[BlockEntity] {
        &self.block_entities
    }

    /// Checks if the block at the given position is transparent to sky light.
    /// This includes air, glass, leaves, and other transparent blocks.
    pub fn is_transparent(&self, position: Coordinates) -> bool {
        self.get_block_state_id(position).is_transparent()
    }

    /// Gets the light level emitted by the block at the given position.
    /// Returns 0 if the block doesn't emit light.
    pub fn get_emitted_light(&self, position: Coordinates) -> u8 {
        self.get_block_state_id(position).get_emitted_light_level()
    }
}
