use crate::handlers::configuration::send_play_packets;
use crate::kick_messages::CLIENT_MODERN_FORWARDING_NOT_SUPPORTED_KICK_MESSAGE;
use crate::server::batch::Batch;
use crate::server::client_state::ClientState;
use crate::server::game_profile::GameProfile;
use crate::server::packet_handler::{PacketHandler, PacketHandlerError};
use crate::server::packet_registry::PacketRegistry;
use crate::server_state::ServerState;
use minecraft_packets::login::custom_query_packet::CustomQueryPacket;
use minecraft_packets::login::game_profile_packet::GameProfilePacket;
use minecraft_packets::login::login_state_packet::LoginStartPacket;
use minecraft_packets::login::login_success_packet::LoginSuccessPacket;
use minecraft_packets::login::set_compression_packet::SetCompressionPacket;
use minecraft_protocol::prelude::ProtocolVersion;
use rand::Rng;

impl PacketHandler for LoginStartPacket {
    fn handle(
        &self,
        client_state: &mut ClientState,
        server_state: &ServerState,
    ) -> Result<Batch<PacketRegistry>, PacketHandlerError> {
        let mut batch = Batch::new();
        if server_state.is_modern_forwarding() {
            if client_state.protocol_version().is_modern() {
                login_start_velocity(&mut batch, client_state);
            } else {
                client_state.kick(CLIENT_MODERN_FORWARDING_NOT_SUPPORTED_KICK_MESSAGE);
            }
        } else {
            let game_profile: GameProfile = self.into();
            fire_login_success(&mut batch, client_state, server_state, game_profile)?;
        }
        Ok(batch)
    }
}

fn login_start_velocity(batch: &mut Batch<PacketRegistry>, client_state: &mut ClientState) {
    let message_id = {
        let mut rng = rand::rng();
        rng.random()
    };
    client_state.set_velocity_login_message_id(message_id);
    let packet = CustomQueryPacket::velocity_info_channel(message_id);
    batch.queue(|| PacketRegistry::CustomQuery(packet));
}

pub fn fire_login_success(
    batch: &mut Batch<PacketRegistry>,
    client_state: &mut ClientState,
    server_state: &ServerState,
    game_profile: GameProfile,
) -> Result<(), PacketHandlerError> {
    let protocol_version = client_state.protocol_version();

    if protocol_version.is_after_inclusive(ProtocolVersion::V1_8)
        && let Some(compression_settings) = server_state.compression_settings()
    {
        let threshold = compression_settings.threshold;
        let packet = SetCompressionPacket::new(i32::try_from(threshold)?);
        batch.queue(|| PacketRegistry::SetCompression(packet));
    }

    if protocol_version.is_after_inclusive(ProtocolVersion::V1_21_2) {
        let packet = LoginSuccessPacket::new(game_profile.uuid(), game_profile.username());
        batch.queue(|| PacketRegistry::LoginSuccess(packet));
    } else {
        let packet = GameProfilePacket::new(game_profile.uuid(), game_profile.username());
        batch.queue(|| PacketRegistry::GameProfile(packet));
    }

    client_state.set_game_profile(game_profile);

    if !protocol_version.supports_configuration_state() {
        send_play_packets(batch, client_state, server_state)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::StreamExt;
    use minecraft_protocol::prelude::{ProtocolVersion, State};

    fn vanilla() -> ServerState {
        ServerState::builder().build().unwrap()
    }

    fn velocity() -> ServerState {
        let mut builder = ServerState::builder();
        let secret = "foo";
        builder.enable_modern_forwarding(secret);
        builder.build().unwrap()
    }

    pub fn client(protocol: ProtocolVersion) -> ClientState {
        let mut cs = ClientState::default();
        cs.set_protocol_version(protocol);
        cs.set_state(State::Login);
        cs
    }

    fn packet() -> LoginStartPacket {
        LoginStartPacket::default()
    }

    // modern forwarding
    #[tokio::test]
    async fn test_login_start_velocity_happy_path() {
        // Given
        let server_state = velocity();
        let mut client_state = client(ProtocolVersion::V1_13); // ≥ 1.13
        let pkt = packet();

        // When
        let batch = pkt.handle(&mut client_state, &server_state).unwrap();
        let mut batch = batch.into_stream();

        // Then
        assert!(
            matches!(batch.next().await.unwrap(), PacketRegistry::CustomQuery(_)),
            "first packet should be the velocity CustomQuery"
        );
        assert_ne!(client_state.get_velocity_login_message_id(), -1);
        assert!(client_state.should_kick().is_none());
        assert!(batch.next().await.is_none());
    }

    #[tokio::test]
    async fn test_login_start_velocity_kicks_old_client() {
        // Given
        let server_state = velocity();
        let mut client_state = client(ProtocolVersion::V1_12_2); // < 1.13
        let pkt = packet();

        // When
        let result = pkt.handle(&mut client_state, &server_state);

        // Then
        assert!(result.is_ok());
        assert_eq!(
            client_state.should_kick(),
            Some(CLIENT_MODERN_FORWARDING_NOT_SUPPORTED_KICK_MESSAGE.to_string())
        );
    }

    // vanilla login
    #[tokio::test]
    async fn test_login_start_vanilla_newer_than_1_21_2() {
        // Given
        let server_state = vanilla();
        let mut client_state = client(ProtocolVersion::V1_21_2);
        let pkt = packet();

        // When
        let batch = pkt.handle(&mut client_state, &server_state).unwrap();
        let mut batch = batch.into_stream();

        // Then
        assert!(
            matches!(batch.next().await.unwrap(), PacketRegistry::LoginSuccess(_)),
            "first packet should be LoginSuccess for ≥ 1.21.2"
        );
    }

    #[tokio::test]
    async fn test_login_start_vanilla_before_1_21_2() {
        // Given
        let server_state = vanilla();
        let mut client_state = client(ProtocolVersion::V1_20_2);
        let pkt = packet();

        // When
        let batch = pkt.handle(&mut client_state, &server_state).unwrap();
        let mut batch = batch.into_stream();

        // Then
        assert!(
            matches!(batch.next().await.unwrap(), PacketRegistry::GameProfile(_)),
            "first packet should be GameProfile for < 1.21.2"
        );
    }

    #[tokio::test]
    async fn test_should_not_send_play_packets_when_configuration_state_was_introduced() {
        // Given
        let server_state = vanilla();
        let mut client_state = client(ProtocolVersion::V1_20_2);
        let pkt = packet();

        // When
        let batch = pkt.handle(&mut client_state, &server_state).unwrap();
        let mut batch = batch.into_stream();

        // Then
        let _ = batch.next().await.unwrap();
        assert!(batch.next().await.is_none());
    }

    #[tokio::test]
    async fn test_should_send_play_packets_for_versions_prior_to_configuration_state() {
        // Given
        let server_state = vanilla();
        let mut client_state = client(ProtocolVersion::V1_20);
        let pkt = packet();

        // When
        let batch = pkt.handle(&mut client_state, &server_state).unwrap();
        let mut batch = batch.into_stream();

        // Then
        let _ = batch.next().await.unwrap();
        assert!(batch.next().await.is_some());
    }
}
