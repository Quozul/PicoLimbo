use minecraft_protocol::prelude::Coordinates;
use pico_nbt::prelude::Nbt;
use pico_text_component::prelude::Component;

pub struct BlockEntityData {
    /// Position within the schematic
    pub position: Coordinates,
    /// (e.g., "minecraft:sign")
    pub block_entity_type: String,
    /// The full NBT data for this block entity
    pub nbt: InternalBlockEntityData,
}

/// Intermediate representation of block entity data.
/// This format sits between the raw schematic NBT and the protocol-specific format.
#[derive(Clone)]
pub enum InternalBlockEntityData {
    Sign {
        sign_block_entity: Box<SignBlockEntity>,
    },
    Generic {
        nbt: Nbt,
    },
}

#[derive(Clone)]
pub struct SignBlockEntity {
    pub front_face: SignFace,
    pub back_face: SignFace,
    pub is_waxed: bool,
}

#[derive(Clone)]
pub struct SignFace {
    pub messages: [Component; 4],
    pub color: String,
    pub is_glowing: bool,
}
