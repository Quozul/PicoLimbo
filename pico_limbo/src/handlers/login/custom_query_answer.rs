use crate::forwarding::check_velocity_key_integrity::{VelocityKeyIntegrity, read_velocity_key};
use crate::handlers::login::login_start::fire_login_success;
use crate::kick_messages::PROXY_REQUIRED_KICK_MESSAGE;
use crate::server::batch::Batch;
use crate::server::client_state::ClientState;
use crate::server::game_profile::GameProfile;
use crate::server::packet_handler::{PacketHandler, PacketHandlerError};
use crate::server::packet_registry::PacketRegistry;
use crate::server_state::ServerState;
use minecraft_packets::login::custom_query_answer_packet::CustomQueryAnswerPacket;
use minecraft_protocol::prelude::BinaryReader;

impl PacketHandler for CustomQueryAnswerPacket {
    fn handle(
        &self,
        client_state: &mut ClientState,
        server_state: &ServerState,
    ) -> Result<Batch<PacketRegistry>, PacketHandlerError> {
        let mut batch = Batch::new();
        let client_message_id = client_state.get_velocity_login_message_id();

        if server_state.is_modern_forwarding() && self.message_id.inner() == client_message_id {
            let secret_key = server_state
                .secret_key()
                .map_err(|_| PacketHandlerError::custom("No secret key"))?;
            let mut reader = BinaryReader::new(&self.data);
            let velocity_key = read_velocity_key(&mut reader, &secret_key);

            match velocity_key {
                VelocityKeyIntegrity::Invalid => {
                    client_state.kick(PROXY_REQUIRED_KICK_MESSAGE);
                }
                VelocityKeyIntegrity::Valid {
                    player_uuid,
                    player_name,
                } => {
                    let game_profile = GameProfile::new(&player_name, player_uuid);
                    fire_login_success(&mut batch, client_state, server_state, game_profile)?;
                }
            }
        }
        Ok(batch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use minecraft_protocol::prelude::{ProtocolVersion, VarInt};

    fn velocity() -> ServerState {
        let mut builder = ServerState::builder();
        builder.enable_modern_forwarding("foo");
        builder.build().unwrap()
    }

    fn client() -> ClientState {
        let mut cs = ClientState::default();
        cs.set_protocol_version(ProtocolVersion::V1_13);
        cs
    }

    fn packet(id: i32, data: Vec<u8>) -> CustomQueryAnswerPacket {
        CustomQueryAnswerPacket {
            message_id: VarInt::new(id),
            is_present: true,
            data,
        }
    }

    #[test]
    fn test_custom_query_answer_kicks_on_invalid_key() {
        // Given
        let server_state = velocity();
        let mut client_state = client();

        let message_id = 42;
        client_state.set_velocity_login_message_id(message_id);

        let pkt = packet(message_id, vec![]);

        // When
        let batch = pkt.handle(&mut client_state, &server_state).unwrap();

        // Then
        assert_eq!(
            client_state.should_kick(),
            Some(PROXY_REQUIRED_KICK_MESSAGE.to_string())
        );
        assert!(batch.into_iter().next().is_none());
    }

    #[test]
    fn test_custom_query_answer_ignored_on_mismatching_id() {
        // Given
        let server_state = velocity();
        let mut client_state = client();
        client_state.set_velocity_login_message_id(10);

        let pkt = packet(11, vec![]);

        // When
        let batch = pkt.handle(&mut client_state, &server_state).unwrap();

        // Then
        assert!(client_state.should_kick().is_none());
        assert!(batch.into_iter().next().is_none());
    }
}
