use crate::forwarding::forwarding_result::LegacyForwardingResult;
use crate::server_state::ServerState;
use minecraft_packets::login::Property;
use minecraft_protocol::prelude::Uuid;
use serde::Deserialize;
use std::str::FromStr;

const TEXTURES_PROPERTY_NAME: &str = "textures";
const BUNGEE_GUARD_TOKEN_PROPERTY_NAME: &str = "bungeeguard-token";

/// `BungeeCord` sends 3 or 4 parts in online mode where Velocity always sends 4
/// Parts are the following: hostname, client IP, player unique ID, properties
/// When using `BungeeGuard`, we must have 4 parts as we need the last one
pub fn check_bungee_cord(state: &ServerState, hostname: &str) -> LegacyForwardingResult {
    if !state.is_legacy_forwarding() && !state.is_bungee_guard_forwarding() {
        return LegacyForwardingResult::NoForwarding;
    }

    let parts: Vec<&str> = hostname.split('\x00').collect();
    let unique_id = parts
        .get(2)
        .map(|unique_id_str| Uuid::from_str(unique_id_str));

    if let Some(Ok(player_uuid)) = unique_id {
        let properties = parts.get(3).and_then(|properties| {
            serde_json::from_str::<Vec<BungeeCordHandshakeProperties>>(properties).ok()
        });

        if state.is_bungee_guard_forwarding() {
            let token_valid = properties
                .as_ref()
                .and_then(|properties| {
                    properties
                        .iter()
                        .find(|p| p.name == BUNGEE_GUARD_TOKEN_PROPERTY_NAME)
                        .and_then(|token| {
                            state
                                .tokens()
                                .ok()
                                .map(|tokens| tokens.contains(&token.value))
                        })
                })
                .unwrap_or(false);

            if !token_valid {
                return LegacyForwardingResult::Invalid;
            }
        }

        let textures = properties.and_then(|properties| {
            properties
                .iter()
                .find(|p| p.name == TEXTURES_PROPERTY_NAME)
                .map(|property| Property::textures(&property.value, property.signature.as_ref()))
        });

        LegacyForwardingResult::Anonymous {
            textures,
            player_uuid,
        }
    } else {
        LegacyForwardingResult::Invalid
    }
}

#[derive(Debug, Deserialize)]
struct BungeeCordHandshakeProperties {
    name: String,
    value: String,
    signature: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn none() -> ServerState {
        ServerState::builder().build().unwrap()
    }

    fn bungee_cord() -> ServerState {
        let mut server_state_builder = ServerState::builder();
        server_state_builder.enable_legacy_forwarding();
        server_state_builder.build().unwrap()
    }

    fn bungee_guard() -> ServerState {
        let mut server_state_builder = ServerState::builder();
        server_state_builder.enable_bungee_guard_forwarding(vec![String::from("the_token")]);
        server_state_builder.build().unwrap()
    }

    #[test]
    fn test_valid_no_forwarding() {
        // Given
        let server_state = none();
        let hostname = "localhost";

        // When
        let validation = check_bungee_cord(&server_state, hostname);

        // Then
        assert!(matches!(validation, LegacyForwardingResult::NoForwarding));
    }

    #[test]
    fn test_offline_bungee_cord_legacy_forwarding() {
        // Given
        let server_state = bungee_cord();
        let hostname = "localhost\x00127.0.0.1\x006856201a9c1f49978608371019daf15e";

        // When
        let validation = check_bungee_cord(&server_state, hostname);

        // Then
        match validation {
            LegacyForwardingResult::Anonymous {
                player_uuid,
                textures,
            } => {
                let expected_player_uuid =
                    Uuid::from_str("6856201a-9c1f-4997-8608-371019daf15e").unwrap();
                assert_eq!(player_uuid, expected_player_uuid);
                assert!(textures.is_none());
            }
            _ => panic!("Expected LegacyForwardingResult::Anonymous"),
        }
    }

    #[test]
    fn test_online_legacy_forwarding_with_properties() {
        // Given
        let server_state = bungee_cord();
        let hostname = "localhost\x00127.0.0.1\x006856201a9c1f49978608371019daf15e\x00[{\"name\":\"textures\",\"value\":\"the_skin_data\"}]";

        // When
        let validation = check_bungee_cord(&server_state, hostname);

        // Then
        match validation {
            LegacyForwardingResult::Anonymous {
                player_uuid,
                textures,
            } => {
                let expected_player_uuid =
                    Uuid::from_str("6856201a-9c1f-4997-8608-371019daf15e").unwrap();
                assert_eq!(player_uuid, expected_player_uuid);
                let textures = textures.unwrap();
                assert_eq!(textures.value(), "the_skin_data");
                assert!(textures.signature().is_none());
            }
            _ => panic!("Expected LegacyForwardingResult::Anonymous"),
        }
    }

    #[test]
    fn test_online_legacy_forwarding_with_signed_properties() {
        // Given
        let server_state = bungee_cord();
        let hostname = "localhost\x00127.0.0.1\x006856201a9c1f49978608371019daf15e\x00[{\"name\":\"textures\",\"value\":\"the_skin_data\",\"signature\":\"the_skin_signature\"}]";

        // When
        let validation = check_bungee_cord(&server_state, hostname);

        // Then
        match validation {
            LegacyForwardingResult::Anonymous {
                player_uuid,
                textures,
            } => {
                let expected_player_uuid =
                    Uuid::from_str("6856201a-9c1f-4997-8608-371019daf15e").unwrap();
                assert_eq!(player_uuid, expected_player_uuid);
                let textures = textures.unwrap();
                assert_eq!(textures.value(), "the_skin_data");
                assert_eq!(textures.signature().unwrap(), "the_skin_signature");
            }
            _ => panic!("Expected LegacyForwardingResult::Anonymous"),
        }
    }

    #[test]
    fn test_invalid_bungee_cord_hostname() {
        // Given
        let server_state = bungee_cord();
        let hostname = "localhost";

        // When
        let validation = check_bungee_cord(&server_state, hostname);

        // Then
        assert!(matches!(validation, LegacyForwardingResult::Invalid));
    }

    #[test]
    fn test_offline_bungee_guard_forwarding() {
        // Given
        let server_state = bungee_guard();
        let hostname = "localhost\x00127.0.0.1\x006856201a9c1f49978608371019daf15e\x00[{\"name\":\"bungeeguard-token\",\"value\":\"the_token\",\"signature\":\"\"}]";

        // When
        let validation = check_bungee_cord(&server_state, hostname);

        // Then
        match validation {
            LegacyForwardingResult::Anonymous {
                player_uuid,
                textures,
            } => {
                let expected_player_uuid =
                    Uuid::from_str("6856201a-9c1f-4997-8608-371019daf15e").unwrap();
                assert_eq!(player_uuid, expected_player_uuid);
                assert!(textures.is_none());
            }
            _ => panic!("Expected LegacyForwardingResult::Anonymous"),
        }
    }

    #[test]
    fn test_invalid_bungee_guard_forwarding() {
        // Given
        let server_state = bungee_guard();
        let hostname = "localhost\x00127.0.0.1\x006856201a9c1f49978608371019daf15e\x00[{\"name\":\"bungeeguard-token\",\"value\":\"other_token\",\"signature\":\"\"}]";

        // When
        let validation = check_bungee_cord(&server_state, hostname);

        // Then
        assert!(matches!(validation, LegacyForwardingResult::Invalid));
    }

    #[test]
    fn test_missing_bungee_guard_forwarding() {
        // Given
        let server_state = bungee_guard();
        let hostname = "localhost\x00127.0.0.1\x006856201a9c1f49978608371019daf15e";

        // When
        let validation = check_bungee_cord(&server_state, hostname);

        // Then
        assert!(matches!(validation, LegacyForwardingResult::Invalid));
    }

    #[test]
    fn test_online_bungee_guard_forwarding_with_properties() {
        // Given
        let server_state = bungee_guard();
        let hostname = "localhost\x00127.0.0.1\x006856201a9c1f49978608371019daf15e\x00[{\"name\":\"textures\",\"value\":\"the_skin_data\"},{\"name\":\"bungeeguard-token\",\"value\":\"the_token\",\"signature\":\"\"}]";

        // When
        let validation = check_bungee_cord(&server_state, hostname);

        // Then
        match validation {
            LegacyForwardingResult::Anonymous {
                player_uuid,
                textures,
            } => {
                let expected_player_uuid =
                    Uuid::from_str("6856201a-9c1f-4997-8608-371019daf15e").unwrap();
                assert_eq!(player_uuid, expected_player_uuid);
                let textures = textures.unwrap();
                assert_eq!(textures.value(), "the_skin_data");
                assert!(textures.signature().is_none());
            }
            _ => panic!("Expected LegacyForwardingResult::Anonymous"),
        }
    }

    #[test]
    fn test_online_bungee_guard_forwarding_with_signed_properties() {
        // Given
        let server_state = bungee_guard();
        let hostname = "localhost\x00127.0.0.1\x006856201a9c1f49978608371019daf15e\x00[{\"name\":\"textures\",\"value\":\"the_skin_data\",\"signature\":\"the_skin_signature\"},{\"name\":\"bungeeguard-token\",\"value\":\"the_token\",\"signature\":\"\"}]";

        // When
        let validation = check_bungee_cord(&server_state, hostname);

        // Then
        match validation {
            LegacyForwardingResult::Anonymous {
                player_uuid,
                textures,
            } => {
                let expected_player_uuid =
                    Uuid::from_str("6856201a-9c1f-4997-8608-371019daf15e").unwrap();
                assert_eq!(player_uuid, expected_player_uuid);
                let textures = textures.unwrap();
                assert_eq!(textures.value(), "the_skin_data");
                assert_eq!(textures.signature().unwrap(), "the_skin_signature");
            }
            _ => panic!("Expected LegacyForwardingResult::Anonymous"),
        }
    }
}
