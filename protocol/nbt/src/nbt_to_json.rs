use crate::nbt::Nbt;
use serde_json::{Value, json};

impl Nbt {
    pub fn to_json(&self) -> Value {
        match self {
            Nbt::End => json!(null),
            Nbt::Byte { name, value } => json!({ name.clone().unwrap_or_default(): *value }),
            Nbt::Short { name, value } => json!({ name.clone().unwrap_or_default(): *value }),
            Nbt::Int { name, value } => json!({ name.clone().unwrap_or_default(): *value }),
            Nbt::Long { name, value } => json!({ name.clone().unwrap_or_default(): *value }),
            Nbt::Float { name, value } => json!({ name.clone().unwrap_or_default(): *value }),
            Nbt::Double { name, value } => json!({ name.clone().unwrap_or_default(): *value }),
            Nbt::ByteArray { name, value } => json!({ name.clone().unwrap_or_default(): value }),
            Nbt::String { name, value } => json!({ name.clone().unwrap_or_default(): value }),
            Nbt::NamelessString { value } => json!({ "": value }),
            Nbt::List { name, value, .. } => {
                let list: Vec<Value> = value.iter().map(|v| v.to_json()).collect();
                json!({ name.clone().unwrap_or_default(): list })
            }
            Nbt::Compound { name, value } => {
                let mut map = serde_json::Map::new();
                for tag in value {
                    if let Value::Object(obj) = tag.to_json() {
                        for (k, v) in obj {
                            map.insert(k, v);
                        }
                    }
                }
                json!({ name.clone().unwrap_or_default(): map })
            }
            Nbt::NamelessCompound { value } => {
                let mut map = serde_json::Map::new();
                for tag in value {
                    if let Value::Object(obj) = tag.to_json() {
                        for (k, v) in obj {
                            map.insert(k, v);
                        }
                    }
                }
                json!(map)
            }
            Nbt::IntArray { name, value } => json!({ name.clone().unwrap_or_default(): value }),
            Nbt::LongArray { name, value } => json!({ name.clone().unwrap_or_default(): value }),
        }
    }
}
