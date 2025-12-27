use pico_identifier::Identifier;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Tag {
    values: Vec<Identifier>,
}

impl Tag {
    pub fn get_values(&self) -> &[Identifier] {
        &self.values
    }
}
