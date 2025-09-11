use crate::server::batch::Batch;
use crate::server::client_state::ClientState;
use crate::server::packet_handler::{PacketHandler, PacketHandlerError};
use crate::server::packet_registry::PacketRegistry;
use crate::server_state::ServerState;
use minecraft_packets::status::ping_request_packet::PingRequestPacket;
use minecraft_packets::status::ping_response_packet::PongResponsePacket;

impl PacketHandler for PingRequestPacket {
    fn handle(
        &self,
        _client_state: &mut ClientState,
        _server_state: &ServerState,
    ) -> Result<Batch<PacketRegistry>, PacketHandlerError> {
        let mut batch = Batch::new();
        let packet = PongResponsePacket {
            timestamp: self.timestamp,
        };
        batch.queue(|| PacketRegistry::PongResponse(packet));
        Ok(batch)
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
        let batch = ping_request_packet
            .handle(&mut client_state, &server_state)
            .unwrap();
        let mut batch = batch.into_iter();

        // Then
        assert!(matches!(
            batch.next().unwrap(),
            PacketRegistry::PongResponse(_)
        ));
        assert!(batch.next().is_none());
    }
}
