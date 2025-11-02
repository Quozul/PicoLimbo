use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct Overrides {
    pub blocks_report: Option<String>,
}
