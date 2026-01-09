use crate::decompress::decompress_gz_file;
use crate::internal_block_entity::BlockEntity;
use blocks_report::{BlockStateLookup, InternalMapping, StateData};
use minecraft_protocol::prelude::{Coordinates, VarInt};
use pico_binutils::prelude::{BinaryReader, BinaryReaderError};
use pico_nbt::prelude::{Nbt, NbtDecodeError};
use std::path::Path;
use thiserror::Error;
use tracing::warn;

#[derive(Error, Debug)]
pub enum SchematicError {
    #[error("Error decompressing or reading file: {0}")]
    Io(#[from] std::io::Error),
    #[error("Error decoding NBT data: {0}")]
    Nbt(#[from] NbtDecodeError),
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
    block_data: Vec<u32>,
    dimensions: Coordinates,
    air_palette_index: u32,
    block_entities: Vec<BlockEntity>,
}

impl Schematic {
    /// Loads a `.schem` file from the given path for a specific Minecraft protocol version.
    pub fn load_schematic_file(
        path: &Path,
        internal_mapping: &InternalMapping,
    ) -> Result<Self, SchematicError> {
        let nbt = Self::load_nbt_from_file(path)?;

        Self::validate_version(&nbt)?;
        let dimensions = Self::extract_dimensions(&nbt)?;
        let (palette, air_palette_index) = Self::get_palette_and_air_index(&nbt, internal_mapping)?;
        let block_data = Self::parse_block_data(&nbt, dimensions, palette.len())?;
        let block_entities = Self::parse_block_entities(&nbt)?;

        Ok(Self {
            palette,
            block_data,
            dimensions,
            air_palette_index,
            block_entities,
        })
    }

    fn load_nbt_from_file(path: &Path) -> Result<Nbt, SchematicError> {
        let bytes = decompress_gz_file(path)?;
        Nbt::from_bytes(&bytes).map_err(Into::into)
    }

    fn validate_version(nbt: &Nbt) -> Result<(), SchematicError> {
        // This handles version 3
        let schematic_version = nbt
            .find_tag("Schematic")
            .and_then(|nbt| nbt.find_tag("Version"))
            .and_then(|nbt| nbt.get_int());
        if let Some(version) = schematic_version {
            return Err(SchematicError::UnsupportedVersion(version));
        }

        // This handles version 1 and 2
        let version = Self::get_tag_as(nbt, "Version", |t| t.get_int())?;
        if version != 2 {
            return Err(SchematicError::UnsupportedVersion(version));
        }
        Ok(())
    }

    fn extract_dimensions(nbt: &Nbt) -> Result<Coordinates, SchematicError> {
        let width = Self::get_tag_as::<i16>(nbt, "Width", |t| t.get_short())? as i32;
        let height = Self::get_tag_as::<i16>(nbt, "Height", |t| t.get_short())? as i32;
        let length = Self::get_tag_as::<i16>(nbt, "Length", |t| t.get_short())? as i32;
        Ok(Coordinates::new(width, height, length))
    }

    fn get_palette_and_air_index(
        nbt: &Nbt,
        internal_mapping: &InternalMapping,
    ) -> Result<(Vec<StateData>, u32), SchematicError> {
        let max_schematic_id = Self::get_tag_as(nbt, "PaletteMax", |t| t.get_int())?;
        let block_state_lookup = BlockStateLookup::new(internal_mapping);

        const AIR_IDENTIFIER: &str = "minecraft:air";
        let internal_air_id = *block_state_lookup
            .parse_state_string(AIR_IDENTIFIER)
            .map_err(|_| SchematicError::AirNotFound)?;

        // Initialize palette with air at index 0
        let mut palette: Vec<StateData> = vec![internal_air_id; (max_schematic_id + 1) as usize];

        let palette_nbt = Self::get_tag_as(nbt, "Palette", |t| t.get_nbt_vec())?;

        for block_tag in palette_nbt {
            if let Some(schematic_palette_id) = block_tag.get_int() {
                let block_name = block_tag.get_name();
                if let Some(name) = block_name.as_ref()
                    && let Ok(state_data) = block_state_lookup.parse_state_string(name)
                    && let Some(entry) = palette.get_mut(schematic_palette_id as usize)
                {
                    *entry = *state_data;
                } else {
                    warn!(
                        "Schematic palette contains ID {} which is greater than PaletteMax of {}. Skipping.",
                        schematic_palette_id, max_schematic_id
                    );
                }
            }
        }

        Ok((palette, 0))
    }

    fn parse_block_data(
        nbt: &Nbt,
        dimensions: Coordinates,
        palette_size: usize,
    ) -> Result<Vec<u32>, SchematicError> {
        let total_blocks = (dimensions.x() * dimensions.y() * dimensions.z()) as usize;
        let block_data_i8 = Self::get_tag_as::<Vec<i8>>(nbt, "BlockData", |t| t.get_byte_array())?;
        let block_data_u8: Vec<u8> = block_data_i8.iter().map(|&b| b as u8).collect();
        let mut reader = BinaryReader::new(&block_data_u8);

        let mut block_data = Vec::with_capacity(total_blocks);

        for _ in 0..total_blocks {
            if reader.remaining() == 0 {
                warn!("Schematic BlockData is smaller than expected dimensions. Truncating.");
                break;
            }

            let schematic_block_id = reader.read::<VarInt>()?.inner() as u32;

            // Validate the index is within palette bounds, use 0 (air) as fallback
            let palette_index = if (schematic_block_id as usize) < palette_size {
                schematic_block_id
            } else {
                warn!(
                    "Block ID {} exceeds palette size {}. Using air as fallback.",
                    schematic_block_id, palette_size
                );
                0
            };

            block_data.push(palette_index);
        }

        // Ensure the vec is the correct size if the data was truncated
        block_data.resize(total_blocks, 0); // Fill with air index
        Ok(block_data)
    }

    fn parse_block_entities(nbt: &Nbt) -> Result<Vec<BlockEntity>, SchematicError> {
        let Some(block_entities_tag) = nbt.find_tag("BlockEntities") else {
            return Ok(Vec::new());
        };

        let Some(block_entities_list) = block_entities_tag.get_nbt_vec() else {
            warn!("BlockEntities tag exists but is not a list. Skipping block entities.");
            return Ok(Vec::new());
        };

        let block_entities = block_entities_list
            .iter()
            .filter_map(BlockEntity::from_nbt)
            .collect::<Vec<BlockEntity>>();

        Ok(block_entities)
    }

    /// Helper function to safely get a required NBT tag and extract its value.
    fn get_tag_as<T>(
        nbt: &Nbt,
        tag_name: &str,
        getter: fn(&Nbt) -> Option<T>,
    ) -> Result<T, SchematicError> {
        nbt.find_tag(tag_name)
            .ok_or_else(|| SchematicError::MissingTag(tag_name.to_string()))
            .and_then(|tag| {
                getter(tag).ok_or_else(|| SchematicError::IncorrectTagType(tag_name.to_string()))
            })
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

    pub fn get_air_id(&self) -> &StateData {
        &self.palette[self.air_palette_index as usize]
    }

    /// Checks if the block at the given position is air
    pub fn is_air(&self, position: Coordinates) -> bool {
        self.get_block_state_id(position).internal_id()
            == self.palette[self.air_palette_index as usize].internal_id()
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
