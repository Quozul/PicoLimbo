use crate::blocks_report::BlocksReports;
use crate::decompress::decompress_gz_file;
use crate::search_block_state::SearchState;
use minecraft_protocol::prelude::{ProtocolVersion, VarInt};
use pico_binutils::prelude::{BinaryReader, BinaryReaderError};
use pico_nbt::prelude::{Nbt, NbtDecodeError};
use std::collections::HashMap;
use std::path::Path;
use thiserror::Error;
use tracing::{debug, warn};

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
    #[error("Failed to initialize block reports")]
    BlocksReportsInit,
}

/// A parsed representation of the schematic's block data and count.
struct ParsedBlockData {
    /// A map from (x, y, z) coordinates to the global block state ID.
    lookup: HashMap<(i32, i32, i32), i32>,
    solid_blocks_count: usize,
}

#[derive(Clone, Default)]
pub struct Schematic {
    block_lookup: HashMap<(i32, i32, i32), i32>,
    dimensions: (i32, i32, i32),
    offset: (i32, i32, i32),
    solid_blocks_count: usize,
}

impl Schematic {
    pub const AIR_BLOCK_STATE_ID: i32 = 0;
    pub const UNKNOWN_BLOCK_STATE_ID: i32 = 1;

    /// Loads a `.schem` file from the given path for a specific Minecraft protocol version.
    pub fn load_schematic_file(
        path: &Path,
        version: ProtocolVersion,
    ) -> Result<Self, SchematicError> {
        let nbt = Self::load_nbt_from_file(path)?;

        Self::validate_version(&nbt)?;
        let dimensions = Self::extract_dimensions(&nbt)?;
        let offset = Self::extract_offset(&nbt)?;

        let block_data = Self::parse_block_data(&nbt, version, dimensions)?;

        Ok(Self {
            block_lookup: block_data.lookup,
            dimensions,
            offset,
            solid_blocks_count: block_data.solid_blocks_count,
        })
    }

    fn load_nbt_from_file(path: &Path) -> Result<Nbt, SchematicError> {
        let bytes = decompress_gz_file(path)?;
        Nbt::from_bytes(&bytes).map_err(Into::into)
    }

    fn validate_version(nbt: &Nbt) -> Result<(), SchematicError> {
        let version = Self::get_tag_as(nbt, "Version", |t| t.get_int())?;
        if version != 2 {
            return Err(SchematicError::UnsupportedVersion(version));
        }

        let data_version = Self::get_tag_as(nbt, "DataVersion", |t| t.get_int())?;
        debug!("Schematic DataVersion: {}", data_version);

        Ok(())
    }

    fn extract_dimensions(nbt: &Nbt) -> Result<(i32, i32, i32), SchematicError> {
        let width = Self::get_tag_as::<i16>(nbt, "Width", |t| t.get_short())? as i32;
        let height = Self::get_tag_as::<i16>(nbt, "Height", |t| t.get_short())? as i32;
        let length = Self::get_tag_as::<i16>(nbt, "Length", |t| t.get_short())? as i32;
        Ok((width, height, length))
    }

    fn extract_offset(nbt: &Nbt) -> Result<(i32, i32, i32), SchematicError> {
        if let Some(offset_tag) = nbt.find_tag("Offset") {
            let offset_array = offset_tag
                .get_int_array()
                .ok_or_else(|| SchematicError::IncorrectTagType("Offset".to_string()))?;
            if offset_array.len() == 3 {
                Ok((offset_array[0], offset_array[1], offset_array[2]))
            } else {
                warn!("'Offset' tag found but has incorrect length, defaulting to (0,0,0).");
                Ok((0, 0, 0))
            }
        } else {
            Ok((0, 0, 0))
        }
    }

    fn parse_block_data(
        nbt: &Nbt,
        mc_version: ProtocolVersion,
        dimensions: (i32, i32, i32),
    ) -> Result<ParsedBlockData, SchematicError> {
        let (width, _, length) = dimensions;
        let blocks_reports = BlocksReports::new().map_err(|_| SchematicError::BlocksReportsInit)?;

        let mut schematic_id_to_global_id = HashMap::new();
        let palette_nbt = Self::get_tag_as(nbt, "Palette", |t| t.get_nbt_vec())?;

        for block_tag in palette_nbt {
            if let Some(schematic_palette_id) = block_tag.get_int() {
                let global_id = block_tag
                    .get_name()
                    .and_then(|name| SearchState::from_string(&blocks_reports, &name))
                    .and_then(|mut search| search.version(mc_version).find(&blocks_reports))
                    .map(|id| id as i32)
                    .unwrap_or(Self::UNKNOWN_BLOCK_STATE_ID);

                schematic_id_to_global_id.insert(schematic_palette_id, global_id);
            }
        }

        let block_data_i8 = Self::get_tag_as::<Vec<i8>>(nbt, "BlockData", |t| t.get_byte_array())?;
        let block_data_u8: Vec<u8> = block_data_i8.iter().map(|&b| b as u8).collect();
        let mut reader = BinaryReader::new(&block_data_u8);

        let mut lookup = HashMap::new();
        let mut solid_blocks_count = 0;
        let total_blocks = dimensions.0 * dimensions.1 * dimensions.2;

        for index in 0..total_blocks {
            if reader.remaining() == 0 {
                warn!("Schematic BlockData is smaller than expected dimensions. Truncating.");
                break;
            }

            let schematic_block_id = reader.read::<VarInt>()?.inner();
            let global_id = schematic_id_to_global_id
                .get(&schematic_block_id)
                .copied()
                .unwrap_or(Self::AIR_BLOCK_STATE_ID);

            if global_id != Self::AIR_BLOCK_STATE_ID {
                let y = index / (width * length);
                let z = (index % (width * length)) / width;
                let x = index % width;
                lookup.insert((x, y, z), global_id);
                solid_blocks_count += 1;
            }
        }

        Ok(ParsedBlockData {
            lookup,
            solid_blocks_count,
        })
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

    fn is_out_of_bounds(&self, x: i32, y: i32, z: i32) -> bool {
        let (max_x, max_y, max_z) = self.dimensions;
        x < 0 || y < 0 || z < 0 || x >= max_x || y >= max_y || z >= max_z
    }

    /// Gets the global block state ID at the given relative coordinates within the schematic.
    /// Returns the ID for air (0) if the coordinates are out of bounds or if the block is air.
    pub fn get_block_state_id(&self, x: i32, y: i32, z: i32) -> i32 {
        if self.is_out_of_bounds(x, y, z) {
            return Self::AIR_BLOCK_STATE_ID;
        }

        self.block_lookup
            .get(&(x, y, z))
            .copied()
            .unwrap_or(Self::AIR_BLOCK_STATE_ID)
    }

    pub fn get_solid_block_count(&self) -> usize {
        self.solid_blocks_count
    }

    pub fn get_dimensions(&self) -> (i32, i32, i32) {
        self.dimensions
    }

    /// Returns the offset of the schematic.
    pub fn get_offset(&self) -> (i32, i32, i32) {
        self.offset
    }
}
