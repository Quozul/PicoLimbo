use minecraft_protocol::prelude::ProtocolVersion;
use pico_nbt::prelude::Nbt;
use pico_text_component::prelude::Component;

#[derive(Clone)]
pub struct SignBlockEntity {
    front_text: SignFace,
    back_text: SignFace,
    is_waxed: bool,
}

impl SignBlockEntity {
    pub fn to_nbt(&self, protocol_version: ProtocolVersion) -> Nbt {
        if protocol_version.is_after_inclusive(ProtocolVersion::V1_20) {
            let front_text = self.front_text.to_nbt("front_text", protocol_version);
            let back_text = self.back_text.to_nbt("back_text", protocol_version);

            Nbt::nameless_compound(vec![
                front_text,
                back_text,
                Nbt::bool("is_waxed", self.is_waxed),
            ])
        } else {
            self.front_text.to_nbt("front_text", protocol_version)
        }
    }

    pub fn from_nbt(entity_nbt: &Nbt) -> Self {
        let is_legacy = entity_nbt.find_tag("Text1").is_some();

        let (front_face, back_face) = if is_legacy {
            let front_face = Self::extract_sign_face_legacy(entity_nbt);
            let back_face = SignFace::default();
            (front_face, back_face)
        } else {
            // Modern format (1.20+)
            let front_face = Self::extract_sign_face(entity_nbt, "front_text");
            let back_face = Self::extract_sign_face(entity_nbt, "back_text");
            (front_face, back_face)
        };

        let is_waxed =
            matches!(entity_nbt.find_tag("is_waxed"), Some(Nbt::Byte { value, .. }) if *value != 0);

        Self {
            front_text: front_face,
            back_text: back_face,
            is_waxed,
        }
    }

    /// Extract sign face from legacy format (1.19 and earlier)
    fn extract_sign_face_legacy(nbt: &Nbt) -> SignFace {
        let mut messages = [
            Component::default(),
            Component::default(),
            Component::default(),
            Component::default(),
        ];
        let mut color = "black".to_string();
        let mut is_glowing = false;

        // Extract color
        if let Some(c) = nbt.find_tag("Color").and_then(|t| t.get_string()) {
            color = c;
        }

        // Extract glowing text
        if let Some(Nbt::Byte { value, .. }) = nbt.find_tag("GlowingText") {
            is_glowing = *value != 0;
        }

        // Extract text lines
        let text_tags = ["Text1", "Text2", "Text3", "Text4"];
        for (i, tag_name) in text_tags.iter().enumerate() {
            if let Some(text_nbt) = nbt.find_tag(tag_name)
                && let Some(text_str) = text_nbt.get_string()
            {
                // Parse JSON text component
                messages[i] =
                    serde_json::from_str(&text_str).unwrap_or_else(|_| Component::new(&text_str));
            }
        }

        SignFace::new(messages, color, is_glowing)
    }

    fn extract_sign_face(nbt: &Nbt, text_side: &str) -> SignFace {
        let mut messages = [
            Component::default(),
            Component::default(),
            Component::default(),
            Component::default(),
        ];
        let mut color = "black".to_string();
        let mut is_glowing = false;

        if let Some(text_tag) = nbt.find_tag(text_side) {
            if let Some(c) = text_tag.find_tag("color").and_then(|t| t.get_string()) {
                color = c;
            }
            if let Some(Nbt::Byte { value, .. }) = text_tag.find_tag("has_glowing_text") {
                is_glowing = *value != 0;
            }
            if let Some(msg_list) = text_tag.find_tag("messages").and_then(|t| t.get_nbt_vec()) {
                for (i, msg) in msg_list.iter().take(4).enumerate() {
                    messages[i] = match msg {
                        Nbt::String { value, .. } => {
                            let text = value
                                .strip_prefix('"')
                                .and_then(|s| s.strip_suffix('"'))
                                .unwrap_or(value);

                            Component::new(text)
                        }
                        _ => Component::from_nbt(msg),
                    };
                }
            }
        }

        SignFace::new(messages, color, is_glowing)
    }
}

#[derive(Clone)]
pub struct SignFace {
    messages: [Component; 4],
    color: String,
    is_glowing: bool,
}

impl Default for SignFace {
    fn default() -> Self {
        Self {
            messages: [
                Component::default(),
                Component::default(),
                Component::default(),
                Component::default(),
            ],
            color: "black".to_string(),
            is_glowing: false,
        }
    }
}

impl SignFace {
    pub fn new(messages: [Component; 4], color: String, is_glowing: bool) -> Self {
        Self {
            messages,
            color,
            is_glowing,
        }
    }

    fn to_nbt(&self, face_name: impl ToString, protocol_version: ProtocolVersion) -> Nbt {
        if protocol_version.is_after_inclusive(ProtocolVersion::V1_20) {
            self.format_sign_text(protocol_version, face_name)
        } else {
            let texts = self.messages.clone().map(|msg| msg.to_json());

            Nbt::nameless_compound(vec![
                Nbt::string("Text1", texts[0].clone()),
                Nbt::string("Text2", texts[1].clone()),
                Nbt::string("Text3", texts[2].clone()),
                Nbt::string("Text4", texts[3].clone()),
                Nbt::string("Color", self.color.clone()),
                Nbt::bool("GlowingText", self.is_glowing),
            ])
        }
    }

    fn format_sign_text(&self, protocol_version: ProtocolVersion, face_name: impl ToString) -> Nbt {
        Nbt::compound(
            face_name,
            vec![
                Nbt::String {
                    name: Some("color".to_string()),
                    value: self.color.clone(),
                },
                Nbt::bool("has_glowing_text", self.is_glowing),
                self.format_messages(protocol_version),
            ],
        )
    }

    fn format_messages(&self, protocol_version: ProtocolVersion) -> Nbt {
        if protocol_version.is_after_inclusive(ProtocolVersion::V1_21_5) {
            Nbt::compound_list(
                "messages",
                self.messages.clone().map(|c| c.to_nbt()).to_vec(),
            )
        } else {
            Nbt::string_list(
                "messages",
                self.messages.clone().map(|c| c.to_json()).to_vec(),
            )
        }
    }
}
