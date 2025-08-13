use pico_nbt::prelude::Nbt;
use serde::Serialize;

#[derive(Serialize)]
pub struct PlainText {
    #[serde(alias = "type")]
    component_type: String,
    text: String,
}

impl Default for PlainText {
    fn default() -> Self {
        Self {
            component_type: "text".into(),
            text: "".into(),
        }
    }
}

impl PlainText {
    pub fn new<S>(content: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            text: content.into(),
            ..Self::default()
        }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }

    pub fn to_nbt(&self) -> Nbt {
        Nbt::string("", &self.text)
    }
}
