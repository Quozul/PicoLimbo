use crate::handlers::configuration::send_message;
use crate::server::batch::Batch;
use crate::server::client_state::ClientState;
use crate::server::packet_handler::{PacketHandler, PacketHandlerError};
use crate::server::packet_registry::PacketRegistry;
use crate::server_state::{Boundaries, ServerState};
use minecraft_packets::play::set_player_position_and_rotation_packet::SetPlayerPositionAndRotationPacket;
use minecraft_packets::play::synchronize_player_position_packet::SynchronizePlayerPositionPacket;

impl PacketHandler for SetPlayerPositionAndRotationPacket {
    fn handle(
        &self,
        client_state: &mut ClientState,
        server_state: &ServerState,
    ) -> Result<Batch<PacketRegistry>, PacketHandlerError> {
        Ok(teleport_player_to_spawn(
            client_state,
            server_state,
            self.feet_y,
        ))
    }
}

pub fn teleport_player_to_spawn(
    client_state: &ClientState,
    server_state: &ServerState,
    feet_y: f64,
) -> Batch<PacketRegistry> {
    let mut batch = Batch::new();
    if let Boundaries::Enabled {
        teleport_message,
        min_y,
    } = server_state.boundaries()
        && feet_y < f64::from(*min_y)
    {
        let (x, y, z) = server_state.spawn_position();
        let packet = SynchronizePlayerPositionPacket::new(x, y, z);
        batch.queue(|| PacketRegistry::SynchronizePlayerPosition(packet));

        if let Some(content) = teleport_message {
            send_message(&mut batch, content, client_state.protocol_version());
        }
    }
    batch
}

#[cfg(test)]
mod tests {
    use super::*;
    use minecraft_protocol::prelude::{ProtocolVersion, State};

    fn server_state_with_min_y(min_y: i32, message: Option<String>) -> ServerState {
        let mut builder = ServerState::builder();
        builder.spawn_position((0.0, 100.0, 0.0));
        if let Some(content) = message {
            builder.boundaries(min_y, content).unwrap();
        } else {
            builder.boundaries(min_y, "").unwrap();
        }

        builder.build().unwrap()
    }

    fn client_state() -> ClientState {
        let mut cs = ClientState::default();
        cs.set_protocol_version(ProtocolVersion::V1_20_5);
        cs.set_state(State::Play);
        cs
    }

    #[test]
    fn test_should_teleport_and_message_player() {
        // Given
        let client_state = client_state();
        let server_state = server_state_with_min_y(0, Some("Direct teleport test".to_string()));

        // When
        let batch = teleport_player_to_spawn(&client_state, &server_state, -1.0);
        let mut batch = batch.into_iter();

        // Then
        assert!(matches!(
            batch.next().unwrap(),
            PacketRegistry::SynchronizePlayerPosition(_)
        ));
        assert!(matches!(
            batch.next().unwrap(),
            PacketRegistry::SystemChatMessage(_) | PacketRegistry::LegacyChatMessage(_)
        ));
        assert!(batch.next().is_none());
    }

    #[test]
    fn test_should_teleport_player() {
        // Given
        let client_state = client_state();
        let server_state = server_state_with_min_y(0, None);

        // When
        let batch = teleport_player_to_spawn(&client_state, &server_state, -1.0);
        let mut batch = batch.into_iter();

        // Then
        assert!(matches!(
            batch.next().unwrap(),
            PacketRegistry::SynchronizePlayerPosition(_)
        ));
        assert!(batch.next().is_none());
    }

    #[test]
    fn test_should_do_nothing() {
        // Given
        let client_state = client_state();
        let server_state = server_state_with_min_y(0, None);

        // When
        let batch = teleport_player_to_spawn(&client_state, &server_state, 10.0);
        let mut batch = batch.into_iter();

        // Then
        assert!(batch.next().is_none());
    }
}
