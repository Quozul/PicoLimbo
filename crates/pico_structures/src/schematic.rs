use crate::decompress::decompress_gz_file;
use crate::internal_block_entity::BlockEntity;
use blocks_report::{BlockStateLookup, InternalId, InternalMapping};
use minecraft_protocol::prelude::{Coordinates, VarInt};
use pico_binutils::prelude::{BinaryReader, BinaryReaderError};
use pico_nbt::prelude::{Nbt, NbtDecodeError};
use std::path::Path;
use thiserror::Error;
use tracing::warn;

/// Blocks that allow sky light to pass through (fully or partially transparent)
const TRANSPARENT_BLOCK_PATTERNS: &[&str] = &[
    "air",
    "cave_air",
    "void_air",
    "glass",
    "leaves",
    "ice",
    "water",
    "lava",
    "barrier",
    "light",
    "structure_void",
    "torch",
    "lantern",
    "chain",
    "iron_bars",
    "glass_pane",
    "fence",
    "wall",
    "slab",
    "stairs",
    "carpet",
    "pressure_plate",
    "button",
    "lever",
    "sign",
    "banner",
    // Flowers and plants
    "flower",
    "sapling",
    "grass",
    "fern",
    "dead_bush",
    "seagrass",
    "kelp",
    "bamboo",
    "sugar_cane",
    "vine",
    "lily_pad",
    "mushroom",
    "wheat",
    "carrots",
    "potatoes",
    "beetroots",
    "melon_stem",
    "pumpkin_stem",
    "cocoa",
    "nether_wart",
    "sweet_berry",
    "chorus_flower",
    "chorus_plant",
    "scaffolding",
    "cobweb",
    "rail",
    "redstone",
    "repeater",
    "comparator",
    "tripwire",
    "string",
    "ladder",
    "snow",
    "fire",
    "soul_fire",
    "campfire",
    "candle",
    "sea_pickle",
    "turtle_egg",
    "frogspawn",
    "hanging_sign",
    "pointed_dripstone",
    "amethyst_cluster",
    "small_amethyst_bud",
    "medium_amethyst_bud",
    "large_amethyst_bud",
    "lightning_rod",
    "end_rod",
    "glow_lichen",
    "sculk_vein",
    "mangrove_roots",
    "azalea",
    "spore_blossom",
    "big_dripleaf",
    "small_dripleaf",
    "moss_carpet",
    "pink_petals",
    "pitcher_plant",
    "pitcher_crop",
    "torchflower",
    "torchflower_crop",
    "decorated_pot",
    "head",
    "skull",
    "door",
    "trapdoor",
    "gate",
    // Additional flowers and plants
    "dandelion",
    "poppy",
    "orchid",
    "allium",
    "tulip",
    "oxeye",
    "cornflower",
    "lily",
    "wither_rose",
    "sunflower",
    "lilac",
    "rose_bush",
    "peony",
    "tall_grass",
    "large_fern",
    "tall_seagrass",
    "pearlescent_froglight",
    "verdant_froglight",
    "ochre_froglight",
    "mangrove_propagule",
    "hanging_roots",
    "cave_vines",
    "twisting_vines",
    "weeping_vines",
    "crimson_roots",
    "warped_roots",
    "nether_sprouts",
    "crop",
    "stem",
    "attached",
    "plant",
    "bush",
    "sprouts",
    "roots",
    "vines",
    // More specific flower names
    "azure_bluet",
    "blue_orchid",
    "red_tulip",
    "orange_tulip",
    "white_tulip",
    "pink_tulip",
    "oxeye_daisy",
    "lily_of_the_valley",
    "wither",
    "rose",
    "daisy",
    "bluet",
    // Pots and decorations
    "potted",
    "pot",
    "flower_pot",
];

/// Blocks that emit light and their light levels
const LIGHT_EMITTING_BLOCKS: &[(&str, u8)] = &[
    ("lantern", 15),
    ("soul_lantern", 10),
    ("campfire", 15),
    ("soul_campfire", 10),
    ("torch", 14),
    ("wall_torch", 14),
    ("soul_torch", 10),
    ("soul_wall_torch", 10),
    ("glowstone", 15),
    ("shroomlight", 15),
    ("sea_lantern", 15),
    ("jack_o_lantern", 15),
    ("redstone_lamp", 15), // When lit
    ("beacon", 15),
    ("end_rod", 14),
    ("fire", 15),
    ("soul_fire", 10),
    ("lava", 15),
    ("magma_block", 3),
    ("crying_obsidian", 10),
    ("respawn_anchor", 15), // When fully charged
    ("glow_lichen", 7),
    ("candle", 3), // 1 candle
    ("candle_cake", 3),
    ("sea_pickle", 6),   // Underwater
    ("redstone_ore", 9), // When touched
    ("deepslate_redstone_ore", 9),
    ("sculk_catalyst", 6),
    ("ochre_froglight", 15),
    ("verdant_froglight", 15),
    ("pearlescent_froglight", 15),
    ("amethyst_cluster", 5),
    ("large_amethyst_bud", 4),
    ("medium_amethyst_bud", 2),
    ("small_amethyst_bud", 1),
    ("brewing_stand", 1),
    ("brown_mushroom", 1),
    ("dragon_egg", 1),
    ("end_portal_frame", 1),
    ("enchanting_table", 7),
    ("ender_chest", 7),
    ("furnace", 13), // When lit
    ("blast_furnace", 13),
    ("smoker", 13),
    ("light", 15), // Light block
];

struct PaletteData {
    schematic_id_to_internal_id: Vec<InternalId>,
    internal_air_id: InternalId,
    transparent_block_ids: Vec<bool>,
    light_emitting_blocks: Vec<u8>,
}

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

#[derive(Default)]
pub struct Schematic {
    /// A flat vector storing all block state IDs, indexed by `y * length * width + z * width + x`.
    block_data: Vec<InternalId>,
    dimensions: Coordinates,
    internal_air_id: InternalId,
    block_entities: Vec<BlockEntity>,
    /// Lookup table for transparent blocks (indexed by InternalId)
    transparent_block_ids: Vec<bool>,
    /// Lookup table for light levels emitted by blocks (indexed by InternalId)
    light_emitting_blocks: Vec<u8>,
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
        let palette_data = Self::get_schematic_id_to_internal_id(&nbt, internal_mapping)?;
        let block_data = Self::parse_block_data(
            &nbt,
            palette_data.schematic_id_to_internal_id,
            dimensions,
            palette_data.internal_air_id,
        )?;
        let block_entities = Self::parse_block_entities(&nbt)?;

        Ok(Self {
            block_data,
            dimensions,
            internal_air_id: palette_data.internal_air_id,
            block_entities,
            transparent_block_ids: palette_data.transparent_block_ids,
            light_emitting_blocks: palette_data.light_emitting_blocks,
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

    fn get_schematic_id_to_internal_id(
        nbt: &Nbt,
        internal_mapping: &InternalMapping,
    ) -> Result<PaletteData, SchematicError> {
        let max_schematic_id = Self::get_tag_as(nbt, "PaletteMax", |t| t.get_int())?;
        let block_state_lookup = BlockStateLookup::new(internal_mapping);

        const AIR_IDENTIFIER: &str = "minecraft:air";
        let internal_air_id = block_state_lookup
            .parse_state_string(AIR_IDENTIFIER)
            .map_err(|_| SchematicError::AirNotFound)?;

        let mut schematic_id_to_internal_id: Vec<InternalId> =
            vec![internal_air_id; (max_schematic_id + 1) as usize];

        const MAX_INTERNAL_ID: usize = u16::MAX as usize + 1;
        let mut transparent_block_ids: Vec<bool> = vec![false; MAX_INTERNAL_ID];
        let mut light_emitting_blocks: Vec<u8> = vec![0; MAX_INTERNAL_ID];
        transparent_block_ids[internal_air_id as usize] = true;

        let palette_nbt = Self::get_tag_as(nbt, "Palette", |t| t.get_nbt_vec())?;

        for block_tag in palette_nbt {
            if let Some(schematic_palette_id) = block_tag.get_int() {
                let block_name = block_tag.get_name();
                let internal_id = block_name
                    .as_ref()
                    .and_then(|name| block_state_lookup.parse_state_string(name).ok())
                    .unwrap_or(internal_air_id);

                if let Some(name) = &block_name {
                    let name_lower = name.to_lowercase();
                    let id_index = internal_id as usize;

                    for pattern in TRANSPARENT_BLOCK_PATTERNS {
                        if name_lower.contains(pattern) {
                            transparent_block_ids[id_index] = true;
                            break;
                        }
                    }

                    for (pattern, light_level) in LIGHT_EMITTING_BLOCKS {
                        if name_lower.contains(pattern) {
                            light_emitting_blocks[id_index] = *light_level;
                            break;
                        }
                    }
                }

                if let Some(entry) =
                    schematic_id_to_internal_id.get_mut(schematic_palette_id as usize)
                {
                    *entry = internal_id;
                } else {
                    warn!(
                        "Schematic palette contains ID {} which is greater than PaletteMax of {}. Skipping.",
                        schematic_palette_id, max_schematic_id
                    );
                }
            }
        }

        Ok(PaletteData {
            schematic_id_to_internal_id,
            internal_air_id,
            transparent_block_ids,
            light_emitting_blocks,
        })
    }

    fn parse_block_data(
        nbt: &Nbt,
        schematic_id_to_internal_id: Vec<InternalId>,
        dimensions: Coordinates,
        fallback_id: InternalId,
    ) -> Result<Vec<InternalId>, SchematicError> {
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

            let schematic_block_id = reader.read::<VarInt>()?.inner();

            let internal_id = schematic_id_to_internal_id
                .get(schematic_block_id as usize)
                .copied()
                .unwrap_or(fallback_id);

            block_data.push(internal_id);
        }

        // Ensure the vec is the correct size if the data was truncated
        block_data.resize(total_blocks, fallback_id);

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
    pub fn get_block_state_id(&self, schematic_position: Coordinates) -> InternalId {
        if self.is_out_of_bounds(&schematic_position) {
            return self.internal_air_id;
        }

        let index = self.position_to_index(schematic_position);

        self.block_data
            .get(index)
            .copied()
            .unwrap_or(self.internal_air_id)
    }

    pub fn get_dimensions(&self) -> Coordinates {
        self.dimensions
    }

    pub fn get_block_entities(&self) -> &[BlockEntity] {
        &self.block_entities
    }

    pub fn get_air_id(&self) -> InternalId {
        self.internal_air_id
    }

    /// Checks if the block at the given position is air
    pub fn is_air(&self, position: Coordinates) -> bool {
        self.get_block_state_id(position) == self.internal_air_id
    }

    /// Checks if the block at the given position is transparent to sky light.
    /// This includes air, glass, leaves, and other transparent blocks.
    pub fn is_transparent(&self, position: Coordinates) -> bool {
        let block_id = self.get_block_state_id(position);
        self.transparent_block_ids
            .get(block_id as usize)
            .copied()
            .unwrap_or(false)
    }

    /// Gets the light level emitted by the block at the given position.
    /// Returns 0 if the block doesn't emit light.
    pub fn get_emitted_light(&self, position: Coordinates) -> u8 {
        let block_id = self.get_block_state_id(position);
        self.light_emitting_blocks
            .get(block_id as usize)
            .copied()
            .unwrap_or(0)
    }
}
