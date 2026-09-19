use crate::block_entities::generic::GenericBlockEntity;
use minecraft_protocol::prelude::ProtocolVersion;
use pico_nbt::{IndexMap, Value};

#[derive(Clone)]
pub struct SignBlockEntity {
    data: IndexMap<String, Value>,
    native_components: bool,
}

impl SignBlockEntity {
    pub fn from_nbt(value: &Value, data_version: Option<i32>) -> pico_nbt::Result<Self> {
        let data = GenericBlockEntity::from_nbt(value);
        let data = data
            .to_nbt()
            .get_compound()
            .ok_or_else(|| pico_nbt::Error::Message("Expected sign compound".into()))?
            .clone();
        Ok(Self {
            data,
            // 1.21.5 stores text components as NBT instead of JSON strings.
            native_components: data_version.is_some_and(|version| version >= 4325),
        })
    }

    pub fn to_version_value(&self, version: ProtocolVersion) -> pico_nbt::Result<Value> {
        let mut data = self.data.clone();
        let modern = data.contains_key("front_text") || data.contains_key("back_text");
        let target_modern = version.is_after_inclusive(ProtocolVersion::V1_20);
        let target_native = version.is_after_inclusive(ProtocolVersion::V1_21_5);
        if target_modern {
            if !modern {
                let mut face = IndexMap::new();
                face.insert(
                    "color".into(),
                    data.swap_remove("Color").unwrap_or_else(|| "black".into()),
                );
                face.insert(
                    "has_glowing_text".into(),
                    data.swap_remove("GlowingText").unwrap_or(Value::Byte(0)),
                );
                for (prefix, key) in [("Text", "messages"), ("FilteredText", "filtered_messages")] {
                    let mut lines = Vec::new();
                    let mut has_lines = false;
                    for line in 1..=4 {
                        let value = data.swap_remove(&format!("{prefix}{line}"));
                        has_lines |= value.is_some();
                        lines.push(value.unwrap_or_else(|| Value::String("\"\"".into())));
                    }
                    if has_lines || key == "messages" {
                        face.insert(key.into(), Value::List(lines));
                    }
                }
                data.insert("front_text".into(), Value::Compound(face));
            }
            for key in ["front_text", "back_text"] {
                let face = data
                    .entry(key.into())
                    .or_insert_with(|| Value::Compound(IndexMap::new()));
                let Value::Compound(face) = face else {
                    return Err(pico_nbt::Error::Message(
                        "Expected sign face compound".into(),
                    ));
                };
                face.entry("color".into()).or_insert_with(|| "black".into());
                face.entry("has_glowing_text".into())
                    .or_insert(Value::Byte(0));
                for key in ["messages", "filtered_messages"] {
                    if key == "filtered_messages" && !face.contains_key(key) {
                        continue;
                    }
                    let messages = face.get(key).and_then(Value::get_list).unwrap_or_default();
                    let lines = (0..4)
                        .map(|line| self.convert_message(messages.get(line), target_native))
                        .collect::<pico_nbt::Result<Vec<_>>>()?;
                    face.insert(key.into(), Value::List(lines));
                }
            }
            data.entry("is_waxed".into()).or_insert(Value::Byte(0));
        } else if modern {
            let front = data.swap_remove("front_text");
            let face = front.as_ref().and_then(Value::get_compound);
            data.swap_remove("back_text");
            data.swap_remove("is_waxed");
            data.insert(
                "Color".into(),
                face.and_then(|face| face.get("color"))
                    .cloned()
                    .unwrap_or_else(|| "black".into()),
            );
            data.insert(
                "GlowingText".into(),
                face.and_then(|face| face.get("has_glowing_text"))
                    .cloned()
                    .unwrap_or(Value::Byte(0)),
            );
            for (key, prefix) in [("messages", "Text"), ("filtered_messages", "FilteredText")] {
                let messages = face
                    .and_then(|face| face.get(key))
                    .and_then(Value::get_list);
                if key == "filtered_messages" && messages.is_none() {
                    continue;
                }
                for line in 0..4 {
                    data.insert(
                        format!("{prefix}{}", line + 1),
                        self.convert_message(
                            messages.and_then(|messages| messages.get(line)),
                            false,
                        )?,
                    );
                }
            }
        } else {
            for line in 1..=4 {
                let key = format!("Text{line}");
                let message = self.convert_message(data.get(&key), false)?;
                data.insert(key, message);
            }
        }
        Ok(Value::Compound(data))
    }

    fn convert_message(
        &self,
        value: Option<&Value>,
        target_native: bool,
    ) -> pico_nbt::Result<Value> {
        let Some(value) = value else {
            return Ok(Value::String(
                if target_native { "" } else { "\"\"" }.into(),
            ));
        };
        if self.native_components == target_native {
            return Ok(value.clone());
        }
        if target_native {
            match value {
                Value::String(json) => match serde_json::from_str(json) {
                    Ok(json) => pico_nbt::json_to_nbt(json),
                    Err(_) => Ok(value.clone()),
                },
                _ => Ok(value.clone()),
            }
        } else {
            let mut json = serde_json::to_value(value)
                .map_err(|error| pico_nbt::Error::Message(error.to_string()))?;
            restore_json_booleans(&mut json);
            Ok(Value::String(json.to_string()))
        }
    }
}

// NBT represents text style booleans as bytes; JSON components require booleans.
fn restore_json_booleans(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Object(fields) => {
            for (key, value) in fields {
                if matches!(
                    key.as_str(),
                    "bold" | "italic" | "underlined" | "strikethrough" | "obfuscated" | "interpret"
                ) && let Some(number) = value.as_i64()
                {
                    *value = serde_json::Value::Bool(number != 0);
                } else {
                    restore_json_booleans(value);
                }
            }
        }
        serde_json::Value::Array(values) => values.iter_mut().for_each(restore_json_booleans),
        _ => {}
    }
}
