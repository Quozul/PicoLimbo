use crate::block_entities::generic::GenericBlockEntity;
use crate::block_entities::sign::SignBlockEntity;
use crate::block_entities::skull::SkullBlockEntity;
use minecraft_protocol::prelude::{Coordinates, ProtocolVersion};
use pico_nbt::Value;
use std::fmt::Display;
use tracing::{debug, warn};

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
            "sign" | "minecraft:sign" => BlockEntityType::Sign,
            "hanging_sign" | "minecraft:hanging_sign" => BlockEntityType::HangingSign,
            other => BlockEntityType::Generic(if other.contains(':') {
                other.to_string()
            } else {
                format!("minecraft:{other}")
            }),
        }
    }
}

#[derive(Clone)]
pub struct BlockEntity {
    pub position: Coordinates,
    pub block_entity_type: BlockEntityType,
    block_entity_data: BlockEntityData,
}

impl BlockEntity {
    pub fn from_nbt(
        entity_nbt: &crate::schematic_file::BlockEntity,
        data_version: Option<i32>,
    ) -> Option<Self> {
        if let Ok(position) = entity_nbt.position() {
            let block_entity_type = BlockEntityType::from(entity_nbt.identifier());
            let value = entity_nbt.data();
            let block_entity_data =
                BlockEntityData::from_nbt(&block_entity_type.to_string(), value, data_version)
                    .map_err(|error| debug!(%error, "Failed to load block entity"))
                    .ok()?;
            Some(Self {
                position,
                block_entity_data,
                block_entity_type,
            })
        } else {
            debug!("Failed to load block entity");
            None
        }
    }

    pub fn to_nbt(&self, protocol_version: ProtocolVersion) -> pico_nbt::Result<Value> {
        self.block_entity_data.value(protocol_version).inspect_err(|error| {
            warn!(%error, entity_type = %self.block_entity_type, "Skipping invalid block entity data");
        })
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
    Skull(SkullBlockEntity),
    Generic { entity: GenericBlockEntity },
}

impl BlockEntityData {
    fn from_nbt(
        id_tag: &str,
        entity_nbt: &Value,
        data_version: Option<i32>,
    ) -> pico_nbt::Result<Self> {
        match id_tag {
            "minecraft:sign" | "minecraft:hanging_sign" => {
                let sign_block_entity = SignBlockEntity::from_nbt(entity_nbt, data_version)?;
                Ok(Self::Sign(Box::new(sign_block_entity)))
            }

            "minecraft:skull" => Ok(Self::Skull(SkullBlockEntity::from_nbt(entity_nbt))),
            _ => Ok(Self::Generic {
                entity: GenericBlockEntity::from_nbt(entity_nbt),
            }),
        }
    }

    pub fn value(&self, protocol_version: ProtocolVersion) -> pico_nbt::Result<Value> {
        match self {
            BlockEntityData::Sign(entity) => entity.to_version_value(protocol_version),
            BlockEntityData::Skull(entity) => Ok(entity.to_version_value(protocol_version)),
            BlockEntityData::Generic { entity } => Ok(entity.to_nbt().clone()),
        }
    }
}
