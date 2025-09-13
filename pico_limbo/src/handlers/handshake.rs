use crate::forwarding::check_bungee_cord::check_bungee_cord;
use crate::forwarding::forwarding_result::LegacyForwardingResult;
use crate::kick_messages::PROXY_REQUIRED_KICK_MESSAGE;
use crate::server::batch::Batch;
use crate::server::client_state::ClientState;
use crate::server::game_profile::GameProfile;
use crate::server::packet_handler::{PacketHandler, PacketHandlerError};
use crate::server::packet_registry::PacketRegistry;
use crate::server_state::ServerState;
use minecraft_packets::handshaking::handshake_packet::HandshakePacket;
use minecraft_protocol::prelude::{ProtocolVersion, State};
use thiserror::Error;

impl PacketHandler for HandshakePacket {
    fn handle(
        &self,
        client_state: &mut ClientState,
        server_state: &ServerState,
    ) -> Result<Batch<PacketRegistry>, PacketHandlerError> {
        let batch = Batch::new();
        client_state.set_protocol_version(self.get_protocol());

        let next_state = self.get_next_state()?;
        client_state.set_state(next_state);

        let forwarding_result = check_bungee_cord(server_state, &self.hostname);
        match forwarding_result {
            LegacyForwardingResult::Invalid => {
                client_state.kick(PROXY_REQUIRED_KICK_MESSAGE);
                Err(PacketHandlerError::ProxyRequired)
            }
            LegacyForwardingResult::Anonymous {
                player_uuid,
                textures,
            } => {
                let game_profile = GameProfile::anonymous(player_uuid, textures);
                client_state.set_game_profile(game_profile);
                Ok(batch)
            }
            LegacyForwardingResult::NoForwarding => Ok(batch),
        }
    }
}

#[derive(Error, Debug)]
#[error("unknown state {0}")]
pub struct UnknownStateError(i32);

trait GetStateProtocol {
    fn get_next_state(&self) -> Result<State, UnknownStateError>;
    fn get_protocol(&self) -> ProtocolVersion;
}

impl GetStateProtocol for HandshakePacket {
    fn get_next_state(&self) -> Result<State, UnknownStateError> {
        let state = self.next_state.inner();
        match state {
            1 => Ok(State::Status),
            2 => Ok(State::Login),
            3 => Ok(State::Transfer),
            _ => Err(UnknownStateError(state)),
        }
    }

    fn get_protocol(&self) -> ProtocolVersion {
        if self.protocol.inner() == -1 {
            ProtocolVersion::Any
        } else {
            ProtocolVersion::from(self.protocol.inner())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use minecraft_protocol::prelude::VarInt;

    fn server_state() -> ServerState {
        ServerState::builder().build().unwrap()
    }

    fn bungee_cord() -> ServerState {
        let mut server_state_builder = ServerState::builder();
        server_state_builder.enable_legacy_forwarding();
        server_state_builder.build().unwrap()
    }

    #[test]
    fn test_handshake_handler_should_update_client_state_to_login() {
        // Given
        let mut client_state = ClientState::default();
        let handshake_packet = HandshakePacket {
            protocol: VarInt::new(-1),
            hostname: String::new(),
            next_state: VarInt::new(2),
            port: 25565,
        };

        // When
        handshake_packet
            .handle(&mut client_state, &server_state())
            .unwrap();

        // Then
        assert_eq!(client_state.state(), State::Login);
    }

    #[test]
    fn test_handshake_handler_should_update_client_state_to_status() {
        // Given
        let mut client_state = ClientState::default();
        let handshake_packet = HandshakePacket {
            protocol: VarInt::new(-1),
            hostname: String::new(),
            next_state: VarInt::new(1),
            port: 25565,
        };

        // When
        handshake_packet
            .handle(&mut client_state, &server_state())
            .unwrap();

        // Then
        assert_eq!(client_state.state(), State::Status);
    }

    #[test]
    fn test_handshake_handler_should_kick_when_received_unknown_state() {
        // Given
        let mut client_state = ClientState::default();
        let handshake_packet = HandshakePacket {
            protocol: VarInt::new(-1),
            hostname: String::new(),
            next_state: VarInt::new(42),
            port: 25565,
        };

        // When
        let result = handshake_packet.handle(&mut client_state, &server_state());

        // Then
        assert!(matches!(
            result,
            Err(PacketHandlerError::UnknownState(UnknownStateError(42)))
        ));
    }

    #[test]
    fn test_handshake_handler_should_update_client_protocol_version() {
        // Given
        let mut client_state = ClientState::default();
        let handshake_packet = HandshakePacket {
            protocol: VarInt::new(578),
            hostname: String::new(),
            next_state: VarInt::new(1),
            port: 25565,
        };

        // When
        handshake_packet
            .handle(&mut client_state, &server_state())
            .unwrap();

        // Then
        assert_eq!(client_state.protocol_version(), ProtocolVersion::V1_15_2);
    }

    #[test]
    fn test_handshake_handler_should_change_state_when_bungee_cord_handshake_is_valid() {
        // Given
        let mut client_state = ClientState::default();
        let handshake_packet = HandshakePacket {
            protocol: VarInt::new(578),
            hostname: "localhost\0127.0.0.1\06856201a9c1f49978608371019daf15e".to_string(),
            next_state: VarInt::new(2),
            port: 25565,
        };

        // When
        handshake_packet
            .handle(&mut client_state, &bungee_cord())
            .unwrap();

        // Then
        assert_eq!(client_state.state(), State::Login);
    }

    #[test]
    fn test_handshake_handler_should_kick_when_bungee_cord_handshake_is_invalid() {
        // Given
        let mut client_state = ClientState::default();
        let handshake_packet = HandshakePacket {
            protocol: VarInt::new(578),
            hostname: String::new(),
            next_state: VarInt::new(2),
            port: 25565,
        };

        // When
        let result = handshake_packet.handle(&mut client_state, &bungee_cord());

        // Then
        assert_eq!(
            client_state.should_kick(),
            Some(PROXY_REQUIRED_KICK_MESSAGE.to_string())
        );
        assert!(matches!(result, Err(PacketHandlerError::ProxyRequired)));
    }
}
