use pico_text_component::prelude::Component;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug)]
pub struct Version {
    pub name: String,
    pub protocol: i32,
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
        version_name: String,
        version_protocol: i32,
        description: &Component,
        online_players: u32,
        max_players: u32,
        enforces_secure_chat: bool,
    ) -> Self {
        let description = serde_json::to_value(description).unwrap();
        StatusResponse {
            version: Version {
                name: version_name,
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
