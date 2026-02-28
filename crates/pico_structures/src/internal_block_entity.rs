use crate::block_entities::generic::GenericBlockEntity;
use crate::block_entities::sign::SignBlockEntity;
use minecraft_protocol::prelude::{Coordinates, ProtocolVersion};
use pico_nbt2::Value;
use std::fmt::Display;
use tracing::debug;

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
    block_entity_data: BlockEntityData,
}

impl BlockEntity {
    pub fn from_nbt(entity_nbt: &crate::schematic_file::BlockEntity) -> Option<Self> {
        if let Ok(position) = entity_nbt.position() {
            let block_entity_type = BlockEntityType::from(entity_nbt.identifier());
            let value = entity_nbt.data();
            let block_entity_data = BlockEntityData::from_nbt(entity_nbt.identifier(), value)
                .expect("Failed to load block entity");
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

    pub fn to_nbt(&self, protocol_version: ProtocolVersion) -> Value {
        self.block_entity_data
            .value(protocol_version)
            .expect("Failed to get Value")
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
    fn from_nbt(id_tag: &str, entity_nbt: &Value) -> pico_nbt2::Result<Self> {
        let entity_nbt = remove_string_tag_quote(entity_nbt);

        match id_tag {
            "minecraft:sign" | "minecraft:hanging_sign" => {
                let sign_block_entity = pico_nbt2::from_value::<SignBlockEntity>(entity_nbt)?;
                Ok(Self::Sign(Box::new(sign_block_entity)))
            }

            _ => Ok(Self::Generic {
                entity: GenericBlockEntity::from_nbt(&entity_nbt),
            }),
        }
    }

    pub fn value(&self, protocol_version: ProtocolVersion) -> pico_nbt2::Result<Value> {
        match self {
            BlockEntityData::Sign(entity) => entity.to_version_value(protocol_version),
            BlockEntityData::Generic { entity } => Ok(entity.to_nbt().clone()),
        }
    }
}

fn remove_string_tag_quote(value: &Value) -> Value {
    match value {
        Value::String(value) => {
            if value.starts_with('"') && value.ends_with('"') {
                Value::String(value[1..value.len() - 1].to_string())
            } else {
                Value::String(value.clone())
            }
        }
        Value::List(values) => Value::List(values.iter().map(remove_string_tag_quote).collect()),
        Value::Compound(values) => Value::Compound(
            values
                .iter()
                .map(|(key, value)| (key.clone(), remove_string_tag_quote(value)))
                .collect(),
        ),
        value => value.clone(),
    }
}
