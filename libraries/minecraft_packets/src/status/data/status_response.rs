use serde::{Deserialize, Serialize};

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
    pub sample: Vec<PlayerSample>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Description {
    pub text: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StatusResponse {
    pub version: Version,
    pub players: Players,
    pub description: Description,
    pub favicon: String,
    #[serde(alias = "enforcesSecureChat")]
    pub enforces_secure_chat: bool,
}

impl StatusResponse {
    pub fn new(
        version_name: impl ToString,
        version_protocol: u32,
        description_text: impl ToString,
        enforces_secure_chat: bool,
    ) -> Self {
        StatusResponse {
            version: Version {
                name: version_name.to_string(),
                protocol: version_protocol,
            },
            players: Players {
                max: 1,
                online: 0,
                sample: Vec::new(),
            },
            description: Description {
                text: description_text.to_string(),
            },
            favicon: String::new(),
            enforces_secure_chat,
        }
    }
}
