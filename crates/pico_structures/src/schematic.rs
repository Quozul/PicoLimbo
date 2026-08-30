use crate::internal_block_entity::BlockEntity;
use crate::schematic_file::SchematicFile;
use blocks_report::{BlockStateLookup, InternalMapping, StateData};
use minecraft_protocol::prelude::Coordinates;
use pico_binutils::prelude::BinaryReaderError;
use std::path::Path;
use thiserror::Error;
use tracing::{debug, warn};

#[derive(Error, Debug)]
pub enum SchematicError {
    #[error("Error decompressing or reading file: {0}")]
    Io(#[from] std::io::Error),
    #[error("Error decoding NBT data: {0}")]
    Nbt(#[from] pico_nbt::Error),
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
    /// State used for out-of-bounds positions and missing palette entries.
    /// The schematic palette may not contain `minecraft:air` at all, so this
    /// cannot be derived from a palette index.
    air: StateData,
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
        let (palette, air) = Self::get_palette_and_air(&schematic_file, internal_mapping)?;
        let block_data = schematic_file.get_block_data().to_vec();
        let block_entities = schematic_file
            .get_block_entities()
            .unwrap_or(&Vec::new())
            .iter()
            .filter_map(BlockEntity::from_nbt)
            .collect::<Vec<_>>();
        debug!("Loaded {} block entities", block_entities.len());

        Ok(Self {
            palette,
            block_data,
            dimensions,
            air,
            block_entities,
        })
    }

    fn get_palette_and_air(
        schematic_file: &SchematicFile,
        internal_mapping: &InternalMapping,
    ) -> Result<(Vec<StateData>, StateData), SchematicError> {
        let max_schematic_id = schematic_file.get_block_palette_max();
        let block_state_lookup = BlockStateLookup::new(internal_mapping);

        const AIR_IDENTIFIER: &str = "minecraft:air";
        let air = *block_state_lookup
            .parse_state_string(AIR_IDENTIFIER)
            .map_err(|_| SchematicError::AirNotFound)?;

        // Initialize palette with air at index 0
        let mut palette: Vec<StateData> = vec![air; max_schematic_id + 1];
        let palette_nbt = schematic_file.get_palette();

        for (block_name, schematic_palette_id) in palette_nbt {
            if let Ok(state_data) = block_state_lookup.parse_state_string(block_name)
                && let Ok(palette_id) = usize::try_from(*schematic_palette_id)
                && let Some(entry) = palette.get_mut(palette_id)
            {
                *entry = *state_data;
            } else {
                warn!(
                    "Schematic palette contains ID {} which is greater than PaletteMax of {}. Skipping.",
                    schematic_palette_id, max_schematic_id
                );
            }
        }

        Ok((palette, air))
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
            return &self.air;
        }

        let index = self.position_to_index(schematic_position);
        self.block_data
            .get(index)
            .and_then(|&palette_index| self.palette.get(palette_index as usize))
            .unwrap_or(&self.air)
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

#[cfg(test)]
mod tests {
    use super::*;
    use flate2::Compression;
    use flate2::write::GzEncoder;
    use std::io::Write;

    /// NBT payload of a 1x1x1 schematic containing a single sea lantern whose
    /// palette does not include `minecraft:air`.
    const SEA_LANTERN_NBT: &[u8] = &[
        0x0A, 0x00, 0x09, 0x53, 0x63, 0x68, 0x65, 0x6D, 0x61, 0x74, 0x69, 0x63, 0x03, 0x00, 0x07,
        0x56, 0x65, 0x72, 0x73, 0x69, 0x6F, 0x6E, 0x00, 0x00, 0x00, 0x02, 0x03, 0x00, 0x0B, 0x44,
        0x61, 0x74, 0x61, 0x56, 0x65, 0x72, 0x73, 0x69, 0x6F, 0x6E, 0x00, 0x00, 0x13, 0x27, 0x0A,
        0x00, 0x08, 0x4D, 0x65, 0x74, 0x61, 0x64, 0x61, 0x74, 0x61, 0x03, 0x00, 0x09, 0x57, 0x45,
        0x4F, 0x66, 0x66, 0x73, 0x65, 0x74, 0x58, 0x00, 0x00, 0x00, 0x00, 0x03, 0x00, 0x09, 0x57,
        0x45, 0x4F, 0x66, 0x66, 0x73, 0x65, 0x74, 0x59, 0xFF, 0xFF, 0xFF, 0xFF, 0x03, 0x00, 0x09,
        0x57, 0x45, 0x4F, 0x66, 0x66, 0x73, 0x65, 0x74, 0x5A, 0x00, 0x00, 0x00, 0x00, 0x0A, 0x00,
        0x09, 0x57, 0x6F, 0x72, 0x6C, 0x64, 0x45, 0x64, 0x69, 0x74, 0x08, 0x00, 0x07, 0x56, 0x65,
        0x72, 0x73, 0x69, 0x6F, 0x6E, 0x00, 0x06, 0x32, 0x2E, 0x31, 0x35, 0x2E, 0x34, 0x08, 0x00,
        0x0F, 0x45, 0x64, 0x69, 0x74, 0x69, 0x6E, 0x67, 0x50, 0x6C, 0x61, 0x74, 0x66, 0x6F, 0x72,
        0x6D, 0x00, 0x18, 0x69, 0x6E, 0x74, 0x65, 0x6C, 0x6C, 0x65, 0x63, 0x74, 0x75, 0x61, 0x6C,
        0x73, 0x69, 0x74, 0x65, 0x73, 0x3A, 0x62, 0x75, 0x6B, 0x6B, 0x69, 0x74, 0x0B, 0x00, 0x06,
        0x4F, 0x66, 0x66, 0x73, 0x65, 0x74, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00, 0x00, 0xFF,
        0xFF, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00, 0x0A, 0x00, 0x09, 0x50, 0x6C, 0x61, 0x74, 0x66,
        0x6F, 0x72, 0x6D, 0x73, 0x0A, 0x00, 0x18, 0x69, 0x6E, 0x74, 0x65, 0x6C, 0x6C, 0x65, 0x63,
        0x74, 0x75, 0x61, 0x6C, 0x73, 0x69, 0x74, 0x65, 0x73, 0x3A, 0x62, 0x75, 0x6B, 0x6B, 0x69,
        0x74, 0x08, 0x00, 0x04, 0x4E, 0x61, 0x6D, 0x65, 0x00, 0x0F, 0x42, 0x75, 0x6B, 0x6B, 0x69,
        0x74, 0x2D, 0x4F, 0x66, 0x66, 0x69, 0x63, 0x69, 0x61, 0x6C, 0x08, 0x00, 0x07, 0x56, 0x65,
        0x72, 0x73, 0x69, 0x6F, 0x6E, 0x00, 0x0E, 0x32, 0x2E, 0x31, 0x35, 0x2E, 0x34, 0x2B, 0x64,
        0x38, 0x36, 0x36, 0x36, 0x62, 0x33, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x05, 0x57, 0x69,
        0x64, 0x74, 0x68, 0x00, 0x01, 0x02, 0x00, 0x06, 0x48, 0x65, 0x69, 0x67, 0x68, 0x74, 0x00,
        0x01, 0x02, 0x00, 0x06, 0x4C, 0x65, 0x6E, 0x67, 0x74, 0x68, 0x00, 0x01, 0x0B, 0x00, 0x06,
        0x4F, 0x66, 0x66, 0x73, 0x65, 0x74, 0x00, 0x00, 0x00, 0x03, 0xFF, 0xFF, 0xFF, 0x68, 0x00,
        0x00, 0x00, 0x4E, 0xFF, 0xFF, 0xFF, 0xD3, 0x03, 0x00, 0x0A, 0x50, 0x61, 0x6C, 0x65, 0x74,
        0x74, 0x65, 0x4D, 0x61, 0x78, 0x00, 0x00, 0x00, 0x01, 0x0A, 0x00, 0x07, 0x50, 0x61, 0x6C,
        0x65, 0x74, 0x74, 0x65, 0x03, 0x00, 0x15, 0x6D, 0x69, 0x6E, 0x65, 0x63, 0x72, 0x61, 0x66,
        0x74, 0x3A, 0x73, 0x65, 0x61, 0x5F, 0x6C, 0x61, 0x6E, 0x74, 0x65, 0x72, 0x6E, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x07, 0x00, 0x09, 0x42, 0x6C, 0x6F, 0x63, 0x6B, 0x44, 0x61, 0x74, 0x61,
        0x00, 0x00, 0x00, 0x01, 0x00, 0x09, 0x00, 0x0D, 0x42, 0x6C, 0x6F, 0x63, 0x6B, 0x45, 0x6E,
        0x74, 0x69, 0x74, 0x69, 0x65, 0x73, 0x0A, 0x00, 0x00, 0x00, 0x00, 0x00,
    ];

    fn write_gzip(data: &[u8], path: &Path) {
        let file = std::fs::File::create(path).unwrap();
        let mut encoder = GzEncoder::new(file, Compression::default());
        encoder.write_all(data).unwrap();
        encoder.finish().unwrap();
    }

    #[test]
    fn out_of_bounds_positions_are_air_when_palette_lacks_air() {
        let internal_mapping = blocks_report::load_internal_mapping().unwrap();
        let lookup = BlockStateLookup::new(&internal_mapping);
        let air_id = lookup
            .parse_state_string("minecraft:air")
            .unwrap()
            .internal_id();
        let lantern_id = lookup
            .parse_state_string("minecraft:sea_lantern")
            .unwrap()
            .internal_id();

        let path = std::env::temp_dir().join("pico_structures_sea_lantern.schem");
        write_gzip(SEA_LANTERN_NBT, &path);
        let schematic = Schematic::load_schematic_file(&path, &internal_mapping).unwrap();
        let _ = std::fs::remove_file(&path);

        let dimensions = schematic.get_dimensions();
        assert_eq!(dimensions.x(), 1);
        assert_eq!(dimensions.y(), 1);
        assert_eq!(dimensions.z(), 1);
        assert_eq!(
            schematic
                .get_block_state_id(Coordinates::new(0, 0, 0))
                .internal_id(),
            lantern_id
        );
        assert_eq!(
            schematic
                .get_block_state_id(Coordinates::new(1, 0, 0))
                .internal_id(),
            air_id
        );
        assert_eq!(
            schematic
                .get_block_state_id(Coordinates::new(15, 15, 15))
                .internal_id(),
            air_id
        );

        // The chunk section covering the schematic must not become a solid
        // cube of the palette block (the reported 16x16x16 duplication bug).
        let world = crate::world::World::from_schematic(&schematic).unwrap();
        let section = world.get_section(&Coordinates::new(0, 0, 0)).unwrap();
        let crate::palette::Palette::Paletted {
            internal_palette, ..
        } = section
        else {
            panic!("expected a paletted section, got a single-value one");
        };
        assert_eq!(internal_palette.len(), 2);
        assert!(internal_palette.contains(&lantern_id));
        assert!(internal_palette.contains(&air_id));
    }
}
