use minecraft_packets::login::Property;
use minecraft_protocol::prelude::Uuid;
use reqwest::Error;
use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct ProfileProperty {
    name: String,
    value: String,
    signature: Option<String>,
}

#[derive(Deserialize)]
pub struct Profile {
    properties: Vec<ProfileProperty>,
}

impl Profile {
    pub fn try_get_textures(&self) -> Option<ProfileProperty> {
        self.properties
            .iter()
            .find(|prop| prop.name == "textures")
            .cloned()
    }
}

pub async fn fetch_minecraft_profile(uuid: Uuid) -> Result<Profile, Error> {
    let uuid_str = uuid.to_string().replace('-', "");
    let url = format!(
        "https://sessionserver.mojang.com/session/minecraft/profile/{uuid_str}?unsigned=false"
    );
    let response = reqwest::get(&url).await?.json::<Profile>().await?;
    Ok(response)
}

impl From<ProfileProperty> for Property {
    fn from(p: ProfileProperty) -> Self {
        Self::new(p.name, p.value, p.signature)
    }
}
