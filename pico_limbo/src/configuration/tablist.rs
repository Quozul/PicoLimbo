use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
#[serde(deny_unknown_fields)]
pub struct TabListConfig {
    /// Header of the tab list displayed when the player presses the tab key.
    pub header: String,

    /// Footer of the tab list displayed when the player presses the tab key.
    pub footer: String,
}
