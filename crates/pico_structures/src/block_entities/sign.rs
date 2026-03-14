use minecraft_protocol::prelude::ProtocolVersion;
use pico_nbt2::{IndexMap, Value, to_value};
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(untagged)]
pub enum Component {
    String(String),
}

impl Default for Component {
    fn default() -> Self {
        Component::String("".to_owned())
    }
}

impl Component {
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    pub fn to_value(&self) -> Value {
        match self {
            Component::String(s) => Value::String(s.to_owned()),
        }
    }
}

/// Wrapper enum for deserializing sign messages in both JSON (pre-1.21.5) and NBT (1.21.5+) formats.
#[derive(Deserialize, Clone)]
#[serde(untagged)]
enum SignMessage {
    /// NBT compound format (1.21.5+)
    Nbt(Component),
    /// JSON string format (pre-1.21.5)
    Json(String),
}

impl SignMessage {
    fn into_component(self) -> Component {
        match self {
            SignMessage::Nbt(c) => c,
            SignMessage::Json(json) => serde_json::from_str(&json).unwrap_or_default(),
        }
    }
}

/// Deserializes a single sign message from either JSON string or NBT compound.
fn deserialize_message<'de, D>(deserializer: D) -> Result<Component, D::Error>
where
    D: Deserializer<'de>,
{
    SignMessage::deserialize(deserializer).map(|msg| msg.into_component())
}

/// Deserializes a vector of sign messages from either JSON strings or NBT compounds.
fn deserialize_messages<'de, D>(deserializer: D) -> Result<Vec<Component>, D::Error>
where
    D: Deserializer<'de>,
{
    Vec::<SignMessage>::deserialize(deserializer).map(|messages| {
        messages
            .into_iter()
            .map(|msg| msg.into_component())
            .collect()
    })
}

#[derive(Default, Deserialize, Serialize, Clone)]
pub enum SignColor {
    #[default]
    #[serde(rename = "black")]
    Black,
    #[serde(rename = "white")]
    White,
    #[serde(rename = "orange")]
    Orange,
    #[serde(rename = "magenta")]
    Magenta,
    #[serde(rename = "light_blue")]
    LightBlue,
    #[serde(rename = "yellow")]
    Yellow,
    #[serde(rename = "lime")]
    Lime,
    #[serde(rename = "pink")]
    Pink,
    #[serde(rename = "gray")]
    Gray,
    #[serde(rename = "light_gray")]
    LightGray,
    #[serde(rename = "cyan")]
    Cyan,
    #[serde(rename = "purple")]
    Purple,
    #[serde(rename = "blue")]
    Blue,
    #[serde(rename = "brown")]
    Brown,
    #[serde(rename = "green")]
    Green,
    #[serde(rename = "red")]
    Red,
}

#[derive(Deserialize, Clone, Default)]
pub struct SignFace {
    #[serde(default)]
    has_glowing_text: i8,
    #[serde(default)]
    color: SignColor,
    #[serde(default, deserialize_with = "deserialize_messages")]
    messages: Vec<Component>,
}

impl SignFace {
    /// Converts this `SignFace` to an NBT `Value`, encoding messages based on protocol version.
    ///
    /// - Before 1.21.5: messages are encoded as JSON strings
    /// - 1.21.5+: messages are encoded as NBT compounds
    pub fn to_value(&self, protocol_version: ProtocolVersion) -> Value {
        let mut map = IndexMap::new();
        map.insert(
            "has_glowing_text".into(),
            Value::Byte(self.has_glowing_text),
        );
        map.insert("color".into(), to_value(&self.color).unwrap());

        let messages: Vec<Value> = if protocol_version.is_after_inclusive(ProtocolVersion::V1_21_5)
        {
            self.messages
                .iter()
                .map(|c: &Component| c.to_value())
                .collect()
        } else {
            self.messages
                .iter()
                .map(|c: &Component| Value::String(c.to_json()))
                .collect()
        };
        map.insert("messages".into(), Value::List(messages));

        Value::Compound(map)
    }
}

#[derive(Deserialize, Clone)]
#[serde(untagged)]
pub enum SignBlockEntity {
    Legacy {
        #[serde(alias = "GlowingText")]
        glowing_text: i8,
        #[serde(alias = "Color")]
        color: SignColor,
        #[serde(alias = "Text1", deserialize_with = "deserialize_message")]
        text_1: Component,
        #[serde(alias = "Text2", deserialize_with = "deserialize_message")]
        text_2: Component,
        #[serde(alias = "Text3", deserialize_with = "deserialize_message")]
        text_3: Component,
        #[serde(alias = "Text4", deserialize_with = "deserialize_message")]
        text_4: Component,
    },
    /// This is the format used starting 1.20
    Modern {
        #[serde(default)]
        is_waxed: i8,
        #[serde(default)]
        front_text: SignFace,
        #[serde(default)]
        back_text: SignFace,
    },
}

impl SignBlockEntity {
    pub fn to_version_value(&self, protocol_version: ProtocolVersion) -> pico_nbt2::Result<Value> {
        if protocol_version.is_after_inclusive(ProtocolVersion::V1_20) {
            let modern = self.to_modern();
            match modern {
                Self::Modern {
                    is_waxed,
                    front_text,
                    back_text,
                } => {
                    let mut map = IndexMap::new();
                    map.insert("is_waxed".into(), Value::Byte(is_waxed));
                    map.insert("front_text".into(), front_text.to_value(protocol_version));
                    map.insert("back_text".into(), back_text.to_value(protocol_version));
                    Ok(Value::Compound(map))
                }
                _ => unreachable!(),
            }
        } else {
            let legacy = self.to_legacy();
            match legacy {
                Self::Legacy {
                    glowing_text,
                    color,
                    text_1,
                    text_2,
                    text_3,
                    text_4,
                } => {
                    let mut map = IndexMap::new();
                    map.insert("GlowingText".into(), Value::Byte(glowing_text));
                    map.insert("Color".into(), to_value(&color)?);
                    map.insert("Text1".into(), Value::String(text_1.to_json()));
                    map.insert("Text2".into(), Value::String(text_2.to_json()));
                    map.insert("Text3".into(), Value::String(text_3.to_json()));
                    map.insert("Text4".into(), Value::String(text_4.to_json()));
                    Ok(Value::Compound(map))
                }
                _ => unreachable!(),
            }
        }
    }

    fn to_legacy(&self) -> Self {
        match self {
            SignBlockEntity::Legacy { .. } => self.clone(),
            SignBlockEntity::Modern { front_text, .. } => {
                let text_1 = front_text.messages.first().cloned().unwrap_or_default();
                let text_2 = front_text.messages.get(1).cloned().unwrap_or_default();
                let text_3 = front_text.messages.get(2).cloned().unwrap_or_default();
                let text_4 = front_text.messages.get(3).cloned().unwrap_or_default();

                SignBlockEntity::Legacy {
                    glowing_text: front_text.has_glowing_text,
                    color: front_text.color.clone(),
                    text_1,
                    text_2,
                    text_3,
                    text_4,
                }
            }
        }
    }

    fn to_modern(&self) -> Self {
        match self {
            SignBlockEntity::Modern { .. } => self.clone(),
            SignBlockEntity::Legacy {
                glowing_text,
                color,
                text_1,
                text_2,
                text_3,
                text_4,
            } => {
                let front_messages = vec![
                    text_1.clone(),
                    text_2.clone(),
                    text_3.clone(),
                    text_4.clone(),
                ];

                SignBlockEntity::Modern {
                    is_waxed: 0,
                    front_text: SignFace {
                        has_glowing_text: *glowing_text,
                        color: color.clone(),
                        messages: front_messages,
                    },
                    back_text: SignFace::default(),
                }
            }
        }
    }
}
