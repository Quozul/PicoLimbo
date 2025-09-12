use minecraft_protocol::prelude::{BinaryWriter, BinaryWriterError, EncodePacket, ProtocolVersion};
use pico_nbt::prelude::Nbt;
use serde::Serialize;

#[derive(Serialize, PartialEq, Debug, Default, Clone)]
pub struct Component {
    #[serde(default)]
    pub text: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,

    #[serde(skip_serializing_if = "is_false", default)]
    pub bold: bool,

    #[serde(skip_serializing_if = "is_false", default)]
    pub italic: bool,

    #[serde(skip_serializing_if = "is_false", default)]
    pub underlined: bool,

    #[serde(skip_serializing_if = "is_false", default)]
    pub strikethrough: bool,

    #[serde(skip_serializing_if = "is_false", default)]
    pub obfuscated: bool,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub extra: Vec<Component>,
}

const fn is_false(b: &bool) -> bool {
    !*b
}

impl Component {
    pub fn new<S>(content: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            text: content.into(),
            ..Default::default()
        }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(&self).unwrap_or_default()
    }

    pub fn to_nbt(&self) -> Nbt {
        let mut compound = vec![Nbt::string("text", &self.text)];

        if let Some(color) = &self.color {
            compound.push(Nbt::string("color", color));
        }

        if self.bold {
            compound.push(Nbt::byte("bold", 1));
        }

        if self.italic {
            compound.push(Nbt::byte("italic", 1));
        }

        if self.underlined {
            compound.push(Nbt::byte("underlined", 1));
        }

        if self.strikethrough {
            compound.push(Nbt::byte("strikethrough", 1));
        }

        if self.obfuscated {
            compound.push(Nbt::byte("obfuscated", 1));
        }

        if !self.extra.is_empty() {
            let mut extras = Vec::with_capacity(self.extra.len());
            for extra in &self.extra {
                extras.push(extra.to_nbt());
            }
            compound.push(Nbt::compound_list("extra", extras));
        }

        Nbt::compound("", compound)
    }
}

impl EncodePacket for Component {
    fn encode(
        &self,
        writer: &mut BinaryWriter,
        protocol_version: ProtocolVersion,
    ) -> Result<(), BinaryWriterError> {
        if protocol_version.is_after_inclusive(ProtocolVersion::V1_20_3) {
            self.to_nbt().encode(writer, protocol_version)?;
        } else {
            self.to_json().encode(writer, protocol_version)?;
        }
        Ok(())
    }
}
