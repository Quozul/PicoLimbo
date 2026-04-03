use md5::{Digest, Md5};
use minecraft_packets::login::Property;
use minecraft_packets::login::login_state_packet::LoginStartPacket;
use minecraft_protocol::prelude::*;
use uuid::Builder as UuidBuilder;

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
    // Matches Java's UUID.nameUUIDFromBytes("OfflinePlayer:<username>" UTF-8 bytes).
    let mut hasher = Md5::new();
    hasher.update(b"OfflinePlayer:");
    hasher.update(username.as_bytes());
    let digest: [u8; 16] = hasher.finalize().into();
    UuidBuilder::from_md5_bytes(digest).into_uuid()
}

#[cfg(test)]
mod tests {
    use super::*;
    use minecraft_packets::login::login_state_packet::LoginStartPacket;
    use minecraft_protocol::prelude::{BinaryReader, BinaryWriter, DecodePacket, EncodePacket};
    use std::str::FromStr;

    #[test]
    fn login_start_with_uuid_keeps_packet_uuid() {
        let expected_uuid =
            Uuid::from_str("01234567-89ab-cdef-0123-456789abcdef").expect("valid uuid");
        let packet = build_login_start_packet("PlayerName", expected_uuid);
        let profile = GameProfile::from(&packet);

        assert_eq!(profile.uuid(), expected_uuid);
    }

    #[test]
    fn login_start_with_nil_uuid_uses_expected_offline_uuid_and_is_idempotent() {
        let packet = build_login_start_packet("PlayerName", Uuid::nil());
        let first = GameProfile::from(&packet);
        let second = GameProfile::from(&packet);
        let expected = Uuid::from_str("823dfbec-453f-3a13-bc3b-1afd172427d6").expect("valid uuid");

        assert_eq!(first.uuid(), expected);
        assert_eq!(second.uuid(), expected);
        assert_eq!(first.uuid(), second.uuid());
    }

    fn build_login_start_packet(name: &str, uuid: Uuid) -> LoginStartPacket {
        let protocol_version = ProtocolVersion::V1_20_2;
        let mut writer = BinaryWriter::new();
        name.to_string()
            .encode(&mut writer, protocol_version)
            .expect("encode name");
        writer.write(&uuid).expect("encode uuid");
        let payload = writer.into_inner();
        let mut reader = BinaryReader::new(&payload);
        let packet =
            LoginStartPacket::decode(&mut reader, protocol_version).expect("decode login start");

        assert_eq!(packet.name(), name);
        packet
    }
}
