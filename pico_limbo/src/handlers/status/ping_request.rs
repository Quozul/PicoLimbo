use crate::server::client_state::ClientState;
use crate::server::packet_handler::{PacketHandler, PacketHandlerError};
use crate::server::packet_registry::PacketRegistry;
use crate::server_state::ServerState;
use minecraft_packets::status::ping_request_packet::PingRequestPacket;
use minecraft_packets::status::ping_response_packet::PongResponsePacket;

impl PacketHandler for PingRequestPacket {
    fn handle(
        &self,
        client_state: &mut ClientState,
        _server_state: &ServerState,
    ) -> Result<(), PacketHandlerError> {
        let packet = PongResponsePacket {
            timestamp: self.timestamp,
        };
        client_state.queue_packet(PacketRegistry::PongResponse(packet));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ping_request_packet() {
        // Given
        let mut client_state = ClientState::default();
        let server_state = ServerState::default();
        let ping_request_packet = PingRequestPacket::default();

        // When
        ping_request_packet
            .handle(&mut client_state, &server_state)
            .unwrap();

        // Then
        assert!(matches!(
            client_state.next_packet().unwrap(),
            PacketRegistry::PongResponse(_)
        ));
        assert!(client_state.has_no_more_packets());
    }
}
