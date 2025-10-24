use crate::decompress::decompress_gz_file;
use crate::internal_block_entity::{
    BlockEntityData, InternalBlockEntityData, SignBlockEntity, SignFace,
};
use blocks_report::{BlockStateLookup, InternalId, InternalMapping};
use minecraft_protocol::prelude::{Coordinates, VarInt};
use pico_binutils::prelude::{BinaryReader, BinaryReaderError};
use pico_nbt::prelude::{Nbt, NbtDecodeError};
use pico_text_component::prelude::Component;
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

#[derive(Default)]
pub struct Schematic {
    /// A flat vector storing all block state IDs, indexed by `y * length * width + z * width + x`.
    block_data: Vec<InternalId>,
    dimensions: Coordinates,
    internal_air_id: InternalId,
    /// Block entities with their positions and NBT data
    block_entities: Vec<BlockEntityData>,
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
        let (schematic_id_to_internal_id, internal_air_id) =
            Self::get_schematic_id_to_internal_id(&nbt, internal_mapping)?;
        let block_data = Self::parse_block_data(
            &nbt,
            schematic_id_to_internal_id,
            dimensions,
            internal_air_id,
        )?;
        let block_entities = Self::parse_block_entities(&nbt)?;

        Ok(Self {
            block_data,
            dimensions,
            internal_air_id,
            block_entities,
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
    ) -> Result<(Vec<InternalId>, InternalId), SchematicError> {
        let max_schematic_id = Self::get_tag_as(nbt, "PaletteMax", |t| t.get_int())?;
        let block_state_lookup = BlockStateLookup::new(internal_mapping);

        const AIR_IDENTIFIER: &str = "minecraft:air";
        let internal_air_id = block_state_lookup
            .parse_state_string(AIR_IDENTIFIER)
            .map_err(|_| SchematicError::AirNotFound)?;

        let mut schematic_id_to_internal_id: Vec<InternalId> =
            vec![internal_air_id; (max_schematic_id + 1) as usize];

        let palette_nbt = Self::get_tag_as(nbt, "Palette", |t| t.get_nbt_vec())?;

        for block_tag in palette_nbt {
            if let Some(schematic_palette_id) = block_tag.get_int() {
                let internal_id = block_tag
                    .get_name()
                    .and_then(|name| block_state_lookup.parse_state_string(&name).ok())
                    .unwrap_or(internal_air_id);

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

        Ok((schematic_id_to_internal_id, internal_air_id))
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

    fn parse_block_entities(nbt: &Nbt) -> Result<Vec<BlockEntityData>, SchematicError> {
        // Skip if no block entities are present
        let Some(block_entities_tag) = nbt.find_tag("BlockEntities") else {
            return Ok(Vec::new());
        };

        let Some(block_entities_list) = block_entities_tag.get_nbt_vec() else {
            warn!("BlockEntities tag exists but is not a list. Skipping block entities.");
            return Ok(Vec::new());
        };

        let mut block_entities = Vec::new();

        for entity_nbt in block_entities_list {
            // Extract position from the Pos tag
            let position = if let Some(pos_tag) = entity_nbt.find_tag("Pos") {
                if let Some(pos_array) = pos_tag.get_int_array() {
                    Coordinates::new(pos_array[0], pos_array[1], pos_array[2])
                } else {
                    warn!("Block entity Pos tag is not an int array. Skipping.");
                    continue;
                }
            } else {
                warn!("Block entity missing Pos tag. Skipping.");
                continue;
            };

            // Extract block entity type
            let block_entity_type = if let Some(id_tag) = entity_nbt.find_tag("Id") {
                if let Some(id_str) = id_tag.get_string() {
                    id_str
                } else {
                    warn!("Block entity Id tag is not a string. Skipping.");
                    continue;
                }
            } else {
                warn!("Block entity missing Id tag. Skipping.");
                continue;
            };

            // Clean the NBT
            let cleaned_nbt = Self::clean_block_entity_nbt(&block_entity_type, &entity_nbt);

            block_entities.push(BlockEntityData {
                position,
                block_entity_type,
                nbt: cleaned_nbt,
            });
        }

        Ok(block_entities)
    }

    /// Converts raw NBT into intermediate format
    fn clean_block_entity_nbt(
        block_entity_type: &str,
        entity_nbt: &Nbt,
    ) -> InternalBlockEntityData {
        match block_entity_type {
            "minecraft:sign" | "minecraft:hanging_sign" => {
                // Check if this is a legacy format sign (1.19 and earlier)
                let is_legacy = entity_nbt.find_tag("Text1").is_some();

                let (front_face, back_face) = if is_legacy {
                    let front_face = Self::extract_sign_face_legacy(entity_nbt);
                    // Back face didn't exist in 1.19 - create empty one
                    let back_face = SignFace {
                        messages: [
                            Component::default(),
                            Component::default(),
                            Component::default(),
                            Component::default(),
                        ],
                        color: "black".to_string(),
                        is_glowing: false,
                    };
                    (front_face, back_face)
                } else {
                    // Modern format (1.20+)
                    let front_face = Self::extract_sign_face(entity_nbt, "front_text");
                    let back_face = Self::extract_sign_face(entity_nbt, "back_text");
                    (front_face, back_face)
                };

                let is_waxed = matches!(entity_nbt.find_tag("is_waxed"), Some(Nbt::Byte { value, .. }) if *value != 0);
                let sign_block_entity = SignBlockEntity {
                    front_face,
                    back_face,
                    is_waxed,
                };
                InternalBlockEntityData::Sign {
                    sign_block_entity: Box::new(sign_block_entity),
                }
            }
            _ => {
                // Remove schematic-specific tags that shouldn't be sent to clients
                let cleaned = match entity_nbt {
                    Nbt::Compound { value, .. } => {
                        let filtered: Vec<Nbt> = value
                            .iter()
                            .filter(|tag| {
                                !matches!(
                                    tag.get_name().as_deref(),
                                    Some("Id" | "Pos" | "x" | "y" | "z" | "keepPacked")
                                )
                            })
                            .cloned()
                            .collect();
                        Nbt::Compound {
                            name: None,
                            value: filtered,
                        }
                    }
                    _ => entity_nbt.clone(),
                };
                InternalBlockEntityData::Generic { nbt: cleaned }
            }
        }
    }

    /// Extract sign face from legacy format (1.19 and earlier)
    fn extract_sign_face_legacy(nbt: &Nbt) -> SignFace {
        let mut messages = [
            Component::default(),
            Component::default(),
            Component::default(),
            Component::default(),
        ];
        let mut color = "black".to_string();
        let mut is_glowing = false;

        // Extract color
        if let Some(c) = nbt.find_tag("Color").and_then(|t| t.get_string()) {
            color = c;
        }

        // Extract glowing text
        if let Some(Nbt::Byte { value, .. }) = nbt.find_tag("GlowingText") {
            is_glowing = *value != 0;
        }

        // Extract text lines
        let text_tags = ["Text1", "Text2", "Text3", "Text4"];
        for (i, tag_name) in text_tags.iter().enumerate() {
            if let Some(text_nbt) = nbt.find_tag(tag_name) {
                if let Some(text_str) = text_nbt.get_string() {
                    // Parse JSON text component
                    messages[i] = serde_json::from_str(&text_str)
                        .unwrap_or_else(|_| Component::new(&text_str));
                }
            }
        }

        SignFace {
            messages,
            color,
            is_glowing,
        }
    }

    fn extract_sign_face(nbt: &Nbt, text_side: &str) -> SignFace {
        let mut messages = [
            Component::default(),
            Component::default(),
            Component::default(),
            Component::default(),
        ];
        let mut color = "black".to_string();
        let mut is_glowing = false;

        if let Some(text_tag) = nbt.find_tag(text_side) {
            if let Some(c) = text_tag.find_tag("color").and_then(|t| t.get_string()) {
                color = c;
            }
            if let Some(Nbt::Byte { value, .. }) = text_tag.find_tag("has_glowing_text") {
                is_glowing = *value != 0;
            }
            if let Some(msg_list) = text_tag.find_tag("messages").and_then(|t| t.get_nbt_vec()) {
                for (i, msg) in msg_list.iter().take(4).enumerate() {
                    messages[i] = match msg {
                        Nbt::String { value, .. } => {
                            let text = value
                                .strip_prefix('"')
                                .and_then(|s| s.strip_suffix('"'))
                                .unwrap_or(value);

                            Component::new(text)
                        }
                        _ => Component::from_nbt(msg),
                    };
                }
            }
        }

        SignFace {
            messages,
            color,
            is_glowing,
        }
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

    pub fn get_block_entities(&self) -> &[BlockEntityData] {
        &self.block_entities
    }
}
