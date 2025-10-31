use crate::block_entities::generic::GenericBlockEntity;
use crate::block_entities::sign::SignBlockEntity;
use minecraft_protocol::prelude::{Coordinates, ProtocolVersion};
use pico_nbt::prelude::Nbt;
use std::fmt::Display;

#[derive(Clone)]
pub enum BlockEntityType {
    Sign,
    HangingSign,
    Generic(String),
}

impl Display for BlockEntityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            BlockEntityType::Sign => "minecraft:sign".to_string(),
            BlockEntityType::HangingSign => "minecraft:hanging_sign".to_string(),
            BlockEntityType::Generic(type_id) => type_id.clone(),
        };
        write!(f, "{str}")
    }
}

impl From<&str> for BlockEntityType {
    fn from(type_id: &str) -> Self {
        match type_id {
            "sign" => BlockEntityType::Sign,
            "minecraft:hanging_sign" => BlockEntityType::HangingSign,
            other => BlockEntityType::Generic(other.to_string()),
        }
    }
}

#[derive(Clone)]
pub struct BlockEntity {
    pub position: Coordinates,
    pub block_entity_type: BlockEntityType,
    pub block_entity_data: BlockEntityData,
}

impl BlockEntity {
    pub fn from_nbt(entity_nbt: &Nbt) -> Option<Self> {
        let coordinates = entity_nbt
            .find_tag("Pos")
            .and_then(|tag| tag.get_int_array())
            .map(|pos_array| Coordinates::new(pos_array[0], pos_array[1], pos_array[2]));
        let id = entity_nbt.find_tag("Id").and_then(|nbt| nbt.get_string());

        if let Some(id_tag) = id
            && let Some(position) = coordinates
        {
            let block_entity_type = BlockEntityType::from(id_tag.as_str());
            let block_entity_data = BlockEntityData::from_nbt(id_tag, entity_nbt);
            Some(Self {
                position,
                block_entity_data,
                block_entity_type,
            })
        } else {
            None
        }
    }

    pub fn to_nbt(&self, protocol_version: ProtocolVersion) -> Nbt {
        self.block_entity_data.to_nbt(protocol_version)
    }

    pub fn get_block_entity_type(&self) -> &BlockEntityType {
        &self.block_entity_type
    }

    pub fn get_position(&self) -> Coordinates {
        self.position
    }
}

#[derive(Clone)]
pub enum BlockEntityData {
    Sign(Box<SignBlockEntity>),
    Generic { entity: GenericBlockEntity },
}

impl BlockEntityData {
    fn from_nbt(id_tag: String, entity_nbt: &Nbt) -> Self {
        match id_tag.as_str() {
            "minecraft:sign" | "minecraft:hanging_sign" => {
                Self::Sign(Box::new(SignBlockEntity::from_nbt(entity_nbt)))
            }

            _ => Self::Generic {
                entity: GenericBlockEntity::from_nbt(entity_nbt),
            },
        }
    }

    fn to_nbt(&self, protocol_version: ProtocolVersion) -> Nbt {
        match self {
            BlockEntityData::Sign(entity) => entity.to_nbt(protocol_version),
            BlockEntityData::Generic { entity } => entity.to_nbt(),
        }
    }
}
