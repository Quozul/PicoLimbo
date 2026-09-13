use crate::block_entities::generic::GenericBlockEntity;
use minecraft_protocol::prelude::{ProtocolVersion, Uuid};
use pico_nbt::{IndexMap, Value};

#[derive(Clone)]
pub struct SkullBlockEntity {
    entity: GenericBlockEntity,
}

impl SkullBlockEntity {
    pub fn from_nbt(value: &Value) -> Self {
        Self {
            entity: GenericBlockEntity::from_nbt(value),
        }
    }

    pub fn to_version_value(&self, version: ProtocolVersion) -> Value {
        let mut data = self.entity.to_nbt().clone();
        let Value::Compound(fields) = &mut data else {
            return data;
        };
        if version.is_after_inclusive(ProtocolVersion::V1_20_5) {
            if !fields.contains_key("profile")
                && let Some(owner) = fields
                    .swap_remove("SkullOwner")
                    .or_else(|| fields.swap_remove("ExtraType"))
            {
                fields.insert("profile".into(), modern_profile(owner));
            }
        } else if let Some(profile) = fields.swap_remove("profile") {
            fields.insert("SkullOwner".into(), legacy_profile(profile));
        }
        let key = if version.is_after_inclusive(ProtocolVersion::V1_20_5) {
            "profile"
        } else {
            "SkullOwner"
        };
        if let Some(Value::Compound(profile)) = fields.get_mut(key) {
            let id_key = if key == "profile" { "id" } else { "Id" };
            if let Some(id) = profile.get_mut(id_key) {
                match &*id {
                    Value::String(string) if version.is_after_inclusive(ProtocolVersion::V1_16) => {
                        if let Ok(uuid) = Uuid::parse_str(string) {
                            *id = Value::IntArray(
                                uuid.as_bytes()
                                    .as_chunks::<4>()
                                    .0
                                    .iter()
                                    .map(|bytes| i32::from_be_bytes(*bytes))
                                    .collect(),
                            );
                        }
                    }
                    Value::IntArray(parts)
                        if !version.is_after_inclusive(ProtocolVersion::V1_16)
                            && parts.len() == 4 =>
                    {
                        let bytes: Vec<_> =
                            parts.iter().flat_map(|part| part.to_be_bytes()).collect();
                        if let Ok(uuid) = Uuid::from_slice(&bytes) {
                            *id = Value::String(uuid.to_string());
                        }
                    }
                    _ => {}
                }
            }
        }
        data
    }
}

fn rename(fields: &mut IndexMap<String, Value>, old: &str, new: &str) {
    if let Some(value) = fields.swap_remove(old) {
        fields.insert(new.into(), value);
    }
}

fn modern_profile(owner: Value) -> Value {
    let mut fields = match owner {
        Value::Compound(fields) => fields,
        Value::String(name) => {
            return Value::Compound(IndexMap::from([("name".into(), name.into())]));
        }
        other => return other,
    };
    rename(&mut fields, "Name", "name");
    rename(&mut fields, "Id", "id");
    if let Some(Value::Compound(properties)) = fields.swap_remove("Properties") {
        let mut converted = Vec::new();
        for (name, entries) in properties {
            if let Value::List(entries) = entries {
                for entry in entries {
                    if let Value::Compound(mut entry) = entry {
                        rename(&mut entry, "Value", "value");
                        rename(&mut entry, "Signature", "signature");
                        entry.insert("name".into(), name.clone().into());
                        converted.push(Value::Compound(entry));
                    }
                }
            }
        }
        fields.insert("properties".into(), Value::List(converted));
    }
    Value::Compound(fields)
}

fn legacy_profile(profile: Value) -> Value {
    let mut fields = match profile {
        Value::Compound(fields) => fields,
        Value::String(name) => {
            return Value::Compound(IndexMap::from([("Name".into(), name.into())]));
        }
        other => return other,
    };
    rename(&mut fields, "name", "Name");
    rename(&mut fields, "id", "Id");
    if let Some(Value::List(properties)) = fields.swap_remove("properties") {
        let mut converted = IndexMap::new();
        for entry in properties {
            if let Value::Compound(mut entry) = entry
                && let Some(Value::String(name)) = entry.swap_remove("name")
            {
                rename(&mut entry, "value", "Value");
                rename(&mut entry, "signature", "Signature");
                let entries = converted
                    .entry(name)
                    .or_insert_with(|| Value::List(Vec::new()));
                if let Value::List(entries) = entries {
                    entries.push(Value::Compound(entry));
                }
            }
        }
        fields.insert("Properties".into(), Value::Compound(converted));
    }
    Value::Compound(fields)
}
