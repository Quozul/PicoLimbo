use minecraft_packets::login::Property;
use minecraft_packets::login::login_state_packet::LoginStartPacket;
use minecraft_protocol::prelude::Uuid;
use sha2::{Digest, Sha256};

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
        let username = value.name();
        let uuid = {
            let login_uuid = value.uuid();
            if login_uuid.is_nil() {
                offline_uuid_from_username(&username)
            } else {
                login_uuid
            }
        };

        Self {
            username,
            uuid,
            textures: None,
        }
    }
}

fn offline_uuid_from_username(username: &str) -> Uuid {
    // Keep UUID stable for offline/legacy clients that do not send one.
    let mut hasher = Sha256::new();
    hasher.update(b"OfflinePlayer:");
    hasher.update(username.as_bytes());
    let digest = hasher.finalize();

    let mut bytes = [0u8; 16];
    bytes.copy_from_slice(&digest[..16]);
    bytes[6] = (bytes[6] & 0x0F) | 0x30; // version 3 style (name-based)
    bytes[8] = (bytes[8] & 0x3F) | 0x80; // RFC4122 variant
    Uuid::from_bytes(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use minecraft_packets::login::login_state_packet::LoginStartPacket;

    #[test]
    fn login_start_without_uuid_gets_stable_fallback_uuid() {
        let mut packet = LoginStartPacket::default();
        packet.name = "PlayerName".to_string();

        let first = GameProfile::from(&packet);
        let second = GameProfile::from(&packet);

        assert!(!first.uuid().is_nil());
        assert_eq!(first.uuid(), second.uuid());
    }
}
