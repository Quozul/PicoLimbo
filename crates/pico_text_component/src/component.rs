use minecraft_protocol::prelude::{BinaryWriter, BinaryWriterError, EncodePacket, ProtocolVersion};
use pico_nbt::prelude::Nbt;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug, Default, Clone)]
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

    pub fn from_nbt(nbt: &Nbt) -> Self {
        let mut component = Component::default();
        if let Some(text) = nbt.find_tag("text").and_then(|n| n.get_string()) {
            component.text = text;
        }
        component.color = nbt.find_tag("color").and_then(|n| n.get_string());
        if let Some(bold) = nbt.find_tag("bold").and_then(|n| n.get_bool()) {
            component.bold = bold;
        }
        if let Some(italic) = nbt.find_tag("italic").and_then(|n| n.get_bool()) {
            component.italic = italic;
        }
        if let Some(underlined) = nbt.find_tag("underlined").and_then(|n| n.get_bool()) {
            component.underlined = underlined;
        }
        if let Some(strikethrough) = nbt.find_tag("strikethrough").and_then(|n| n.get_bool()) {
            component.strikethrough = strikethrough;
        }
        if let Some(obfuscated) = nbt.find_tag("obfuscated").and_then(|n| n.get_bool()) {
            component.obfuscated = obfuscated;
        }
        component
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

    pub fn to_legacy(&self) -> String {
        #[derive(Serialize)]
        struct TextComponent {
            #[serde(default)]
            text: String,
        }
        serde_json::to_string(&TextComponent {
            text: self.to_legacy_impl(true),
        })
        .unwrap_or_default()
    }

    fn to_legacy_impl(&self, is_root: bool) -> String {
        let mut s = String::new();

        if !is_root {
            s.push('§');
            s.push('r');
        }

        if let Some(color) = &self.color {
            let color_letter = match color.as_str() {
                "black" => '0',
                "dark_blue" => '1',
                "dark_green" => '2',
                "dark_aqua" => '3',
                "dark_red" => '4',
                "dark_purple" => '5',
                "gold" => '6',
                "gray" => '7',
                "dark_gray" => '8',
                "blue" => '9',
                "green" => 'a',
                "aqua" => 'b',
                "red" => 'c',
                "light_purple" => 'd',
                "yellow" => 'e',
                "white" => 'f',
                _ => 'f',
            };
            s.push('§');
            s.push(color_letter);
        }

        if self.bold {
            s.push('§');
            s.push('l');
        }
        if self.italic {
            s.push('§');
            s.push('o');
        }
        if self.underlined {
            s.push('§');
            s.push('n');
        }
        if self.strikethrough {
            s.push('§');
            s.push('m');
        }
        if self.obfuscated {
            s.push('§');
            s.push('k');
        }

        s.push_str(&self.text);

        for extra in &self.extra {
            s.push_str(&extra.to_legacy_impl(false));
        }

        s
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
