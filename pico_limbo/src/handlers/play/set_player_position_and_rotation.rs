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
    ) -> Result<(), PacketHandlerError> {
        teleport_player_to_spawn(client_state, server_state, self.feet_y);
        Ok(())
    }
}

pub fn teleport_player_to_spawn(
    client_state: &mut ClientState,
    server_state: &ServerState,
    feet_y: f64,
) {
    if let Boundaries::Enabled {
        teleport_message,
        min_y,
    } = server_state.boundaries()
        && feet_y < f64::from(*min_y)
    {
        let (x, y, z) = server_state.spawn_position();
        let packet = SynchronizePlayerPositionPacket::new(x, y, z);
        client_state.queue_packet(PacketRegistry::SynchronizePlayerPosition(packet));

        if let Some(content) = teleport_message {
            client_state.send_message(content);
        }
    }
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

    fn create_packet(feet_y: f64) -> SetPlayerPositionAndRotationPacket {
        SetPlayerPositionAndRotationPacket {
            x: 25.0,
            feet_y,
            z: 35.0,
            yaw: 90.0,
            pitch: 45.0,
            v1_21_4_flags: 0,
            on_ground: false,
        }
    }

    #[test]
    fn test_player_above_min_y_pos_no_packets_sent() {
        // Given
        let mut client_state = client_state();
        let server_state = server_state_with_min_y(30, None);
        let packet = create_packet(40.0);

        // When
        packet.handle(&mut client_state, &server_state).unwrap();

        // Then
        assert!(client_state.has_no_more_packets());
    }

    #[test]
    fn test_player_below_min_y_pos_teleport_packet_sent() {
        // Given
        let mut client_state = client_state();
        let server_state = server_state_with_min_y(30, None);
        let packet = create_packet(20.0);

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
        let server_state = server_state_with_min_y(30, Some("You fell too far!".to_string()));
        let packet = create_packet(20.0);

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
        let server_state = server_state_with_min_y(30, None);
        let packet = create_packet(30.0);

        // When
        packet.handle(&mut client_state, &server_state).unwrap();

        // Then
        assert!(client_state.has_no_more_packets());
    }

    #[test]
    fn test_player_slightly_below_min_y_pos() {
        // Given
        let mut client_state = client_state();
        let server_state = server_state_with_min_y(30, None);
        let packet = create_packet(29.999);

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
    fn test_with_empty_message_only_teleport_sent() {
        // Given
        let mut client_state = client_state();
        let server_state = server_state_with_min_y(30, Some(String::new())); // Empty message
        let packet = create_packet(20.0); // Below min_y_pos

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
    fn test_teleport_player_to_spawn_function_directly() {
        // Given
        let mut client_state = client_state();
        let server_state = server_state_with_min_y(0, Some("Direct teleport test".to_string()));

        // When
        teleport_player_to_spawn(&mut client_state, &server_state, -1.0);

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
    fn test_multiple_protocol_versions_message_handling() {
        // Given
        let mut client_state_legacy = ClientState::default();
        client_state_legacy.set_protocol_version(ProtocolVersion::V1_18_2);
        client_state_legacy.set_state(State::Play);

        let server_state = server_state_with_min_y(30, Some("Legacy message".to_string()));
        let packet = create_packet(20.0);

        // When
        packet
            .handle(&mut client_state_legacy, &server_state)
            .unwrap();

        // Then
        assert!(matches!(
            client_state_legacy.next_packet(),
            PacketRegistry::SynchronizePlayerPosition(_)
        ));
        assert!(matches!(
            client_state_legacy.next_packet(),
            PacketRegistry::LegacyChatMessage(_)
        ));
        assert!(client_state_legacy.has_no_more_packets());

        let mut client_state_modern = ClientState::default();
        client_state_modern.set_protocol_version(ProtocolVersion::V1_20_2);
        client_state_modern.set_state(State::Play);

        let packet = create_packet(20.0);
        packet
            .handle(&mut client_state_modern, &server_state)
            .unwrap();

        assert!(matches!(
            client_state_modern.next_packet(),
            PacketRegistry::SynchronizePlayerPosition(_)
        ));
        assert!(matches!(
            client_state_modern.next_packet(),
            PacketRegistry::SystemChatMessage(_)
        ));
        assert!(client_state_modern.has_no_more_packets());
    }
}
