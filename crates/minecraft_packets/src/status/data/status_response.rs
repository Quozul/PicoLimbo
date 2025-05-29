use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug)]
pub struct Version {
    pub name: String,
    pub protocol: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PlayerSample {
    pub name: String,
    pub id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Players {
    pub max: u32,
    pub online: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sample: Option<Vec<PlayerSample>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StatusResponse {
    pub version: Version,
    pub players: Players,
    pub description: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub favicon: Option<String>,
    #[serde(
        alias = "enforcesSecureChat",
        default = "get_default_enforces_secure_chat"
    )]
    pub enforces_secure_chat: bool,
}

fn get_default_enforces_secure_chat() -> bool {
    false
}

impl StatusResponse {
    pub fn new(
        version_name: &str,
        version_protocol: u32,
        description_text: &str,
        online_players: u32,
        max_players: u32,
        enforces_secure_chat: bool,
    ) -> Self {
        let mut description_map = serde_json::Map::new();
        description_map.insert(
            "text".to_string(),
            Value::String(description_text.to_string()),
        );
        let description = Value::Object(description_map);
        StatusResponse {
            version: Version {
                name: version_name.to_string(),
                protocol: version_protocol,
            },
            players: Players {
                max: max_players,
                online: online_players,
                sample: None,
            },
            description,
            favicon: None,
            enforces_secure_chat,
        }
    }
}
