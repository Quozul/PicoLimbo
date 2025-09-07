use crate::handlers::play::set_player_position_and_rotation::teleport_player_to_spawn;
use crate::server::client_state::ClientState;
use crate::server::packet_handler::{PacketHandler, PacketHandlerError};
use crate::server_state::ServerState;
use minecraft_packets::play::set_player_position_packet::SetPlayerPositionPacket;

impl PacketHandler for SetPlayerPositionPacket {
    fn handle(
        &self,
        client_state: &mut ClientState,
        server_state: &ServerState,
    ) -> Result<(), PacketHandlerError> {
        teleport_player_to_spawn(client_state, server_state, self.feet_y);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::packet_registry::PacketRegistry;
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

    fn create_packet(feet_y: f64) -> SetPlayerPositionPacket {
        SetPlayerPositionPacket {
            x: 10.0,
            feet_y,
            z: 20.0,
            v1_21_4_flags: 0,
            on_ground: true,
        }
    }

    #[test]
    fn test_player_above_min_y_pos_no_packets_sent() {
        // Given
        let mut client_state = client_state();
        let server_state = server_state_with_min_y(50, None);
        let packet = create_packet(60.0);

        // When
        packet.handle(&mut client_state, &server_state).unwrap();

        // Then
        assert!(client_state.has_no_more_packets());
    }

    #[test]
    fn test_player_below_min_y_pos_teleport_packet_sent() {
        // Given
        let mut client_state = client_state();
        let server_state = server_state_with_min_y(50, None);
        let packet = create_packet(40.0);

        // When
        packet.handle(&mut client_state, &server_state).unwrap();

        // Then
        assert!(matches!(
            client_state.next_packet(),
            PacketRegistry::SynchronizePlayerPosition(_)
        ));
        assert!(client_state.has_no_more_packets());
    }

    #[test]
    fn test_player_below_min_y_pos_with_message_two_packets_sent() {
        // Given
        let mut client_state = client_state();
        let server_state = server_state_with_min_y(50, Some("You are too low!".to_string()));
        let packet = create_packet(40.0);

        // When
        packet.handle(&mut client_state, &server_state).unwrap();

        // Then
        assert!(matches!(
            client_state.next_packet(),
            PacketRegistry::SynchronizePlayerPosition(_)
        ));
        let second_packet = client_state.next_packet();
        assert!(matches!(
            second_packet,
            PacketRegistry::SystemChatMessage(_) | PacketRegistry::LegacyChatMessage(_)
        ));
        assert!(client_state.has_no_more_packets());
    }

    #[test]
    fn test_player_exactly_at_min_y_pos_no_packets_sent() {
        // Given
        let mut client_state = client_state();
        let server_state = server_state_with_min_y(50, None);
        let packet = create_packet(50.0); // Exactly at min_y_pos (50)

        // When
        packet.handle(&mut client_state, &server_state).unwrap();

        // Then
        assert!(client_state.has_no_more_packets());
    }

    #[test]
    fn test_player_slightly_below_min_y_pos() {
        // Given
        let mut client_state = client_state();
        let server_state = server_state_with_min_y(50, None);
        let packet = create_packet(49.9);

        // When
        packet.handle(&mut client_state, &server_state).unwrap();

        // Then
        assert!(matches!(
            client_state.next_packet(),
            PacketRegistry::SynchronizePlayerPosition(_)
        ));
        assert!(client_state.has_no_more_packets());
    }

    #[test]
    fn test_negative_min_y_pos() {
        // Given
        let mut client_state = client_state();
        let server_state = server_state_with_min_y(-64, None);
        let packet = create_packet(-70.0);

        // When
        packet.handle(&mut client_state, &server_state).unwrap();

        // Then
        assert!(matches!(
            client_state.next_packet(),
            PacketRegistry::SynchronizePlayerPosition(_)
        ));
        assert!(client_state.has_no_more_packets());
    }
}
