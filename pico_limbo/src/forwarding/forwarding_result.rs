use minecraft_packets::login::Property;
use minecraft_protocol::prelude::Uuid;

pub enum ModernForwardingResult {
    Invalid,
    Valid {
        player_uuid: Uuid,
        player_name: String,
        textures: Option<Property>,
    },
}

pub enum LegacyForwardingResult {
    Invalid,
    Anonymous {
        player_uuid: Uuid,
        textures: Option<Property>,
    },
    NoForwarding,
}
