use crate::server::packet_handler::PacketHandlerError;
use crate::server_state::{MisconfiguredForwardingError, ServerState};
use serde::Deserialize;
use serde_json::Error as JsonError;
use thiserror::Error;

pub fn check_bungee_cord(state: &ServerState, hostname: &str) -> Result<bool, BungeeGuardError> {
    if state.is_legacy_forwarding() {
        let part_count = hostname.split('\0').count();
        // BungeeCord sends 3 or 4 parts in online mode where Velocity always sends 4
        // Parts are the following: hostname, client IP, player unique ID, properties
        Ok(part_count == 3 || part_count == 4)
    } else if state.is_bungee_guard_forwarding() {
        check_bungee_guard_token(state, hostname)
    } else {
        Ok(true) // No forwarding method enabled
    }
}

fn check_bungee_guard_token(state: &ServerState, hostname: &str) -> Result<bool, BungeeGuardError> {
    const BUNGEE_GUARD_TOKEN_PROPERTY_NAME: &str = "bungeeguard-token";
    let parts: Vec<&str> = hostname.split('\0').collect();

    // When using BungeeGuard, we must have 4 parts as we need the last one
    if parts.len() != 4 {
        return Ok(false);
    }

    let properties: Vec<BungeeCordHandshakeProperties> = serde_json::from_str(parts[3])?;

    let token_valid = properties
        .iter()
        .find(|p| p.name == BUNGEE_GUARD_TOKEN_PROPERTY_NAME)
        .map(|token| state.tokens().map(|tokens| tokens.contains(&token.value)))
        .transpose()?
        .unwrap_or(false);

    Ok(token_valid)
}

#[derive(Debug, Deserialize)]
struct BungeeCordHandshakeProperties {
    name: String,
    value: String,
}

#[derive(Debug, Error)]
pub enum BungeeGuardError {
    #[error("invalid json")]
    InvalidJson(#[from] JsonError),
    #[error("misconfigured forwarding")]
    MissingTokens(#[from] MisconfiguredForwardingError),
}

impl From<BungeeGuardError> for PacketHandlerError {
    fn from(e: BungeeGuardError) -> Self {
        Self::custom(&e.to_string())
    }
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

    #[test]
    fn test_valid_no_forwarding() {
        // Given
        let server_state = none();
        let hostname = "localhost";

        // When
        let validation = check_bungee_cord(&server_state, hostname);

        // Then
        assert!(validation.unwrap());
    }

    #[test]
    fn test_valid_bungee_cord_ip_forwarding_hostname() {
        // Given
        let server_state = bungee_cord();
        let hostname = "localhost\0client_ip\0player_uuid";

        // When
        let validation = check_bungee_cord(&server_state, hostname);

        // Then
        assert!(validation.unwrap());
    }

    #[test]
    fn test_valid_velocity_legacy_hostname() {
        // Given
        let server_state = bungee_cord();
        let hostname = "localhost\0client_ip\0player_uuid\0properties";

        // When
        let validation = check_bungee_cord(&server_state, hostname);

        // Then
        assert!(validation.unwrap());
    }

    #[test]
    fn test_invalid_bungee_cord_hostname() {
        // Given
        let server_state = bungee_cord();
        let hostname = "localhost";

        // When
        let validation = check_bungee_cord(&server_state, hostname);

        // Then
        assert!(!validation.unwrap());
    }
}
