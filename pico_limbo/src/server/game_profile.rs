use minecraft_packets::login::Property;
use minecraft_packets::login::login_state_packet::LoginStartPacket;
use minecraft_protocol::prelude::Uuid;

#[derive(Clone)]
pub struct GameProfile {
    username: String,
    uuid: Uuid,
    textures: Option<Property>,
}

impl GameProfile {
    pub fn new(username: &str, uuid: Uuid, textures: Option<Property>) -> Self {
        let username = username
            .get(..16)
            .map_or_else(|| username.to_string(), std::string::ToString::to_string);
        Self {
            username,
            uuid,
            textures,
        }
    }

    pub const fn anonymous(uuid: Uuid, textures: Option<Property>) -> Self {
        Self {
            username: String::new(),
            uuid,
            textures,
        }
    }

    pub fn username(&self) -> &str {
        &self.username
    }

    pub const fn is_anonymous(&self) -> bool {
        self.username.is_empty()
    }

    pub fn set_name<S>(&mut self, name: &S)
    where
        S: ToString,
    {
        self.username = name.to_string();
    }

    pub const fn uuid(&self) -> Uuid {
        self.uuid
    }

    pub const fn textures(&self) -> Option<&Property> {
        self.textures.as_ref()
    }
}

impl From<&LoginStartPacket> for GameProfile {
    fn from(value: &LoginStartPacket) -> Self {
        Self {
            username: value.name(),
            uuid: value.uuid(),
            textures: None,
        }
    }
}
