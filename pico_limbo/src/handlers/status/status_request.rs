use crate::server::batch::Batch;
use crate::server::client_state::ClientState;
use crate::server::packet_handler::{PacketHandler, PacketHandlerError};
use crate::server::packet_registry::PacketRegistry;
use crate::server_state::ServerState;
use minecraft_packets::status::data::status_response::StatusResponse;
use minecraft_packets::status::status_request_packet::StatusRequestPacket;
use minecraft_packets::status::status_response_packet::StatusResponsePacket;
use minecraft_protocol::prelude::ProtocolVersion;

impl PacketHandler for StatusRequestPacket {
    fn handle(
        &self,
        client_state: &mut ClientState,
        server_state: &ServerState,
    ) -> Result<Batch<PacketRegistry>, PacketHandlerError> {
        let mut batch = Batch::new();
        let client_protocol_version = client_state.protocol_version();
        let (version_string, version_number) = if client_protocol_version.is_any() {
            let oldest = ProtocolVersion::oldest();
            let latest = ProtocolVersion::latest();
            let version_string = format!("{oldest}-{latest}");
            (version_string, -1)
        } else {
            (
                client_protocol_version.humanize().to_string(),
                client_protocol_version.version_number(),
            )
        };

        let status_response = StatusResponse::new(
            version_string,
            version_number,
            server_state.motd(),
            server_state.online_players(),
            server_state.max_players(),
            server_state.fav_icon(),
        );
        let packet = StatusResponsePacket::from_status_response(&status_response);
        batch.queue(|| PacketRegistry::StatusResponse(packet));
        Ok(batch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::StreamExt;
    use minecraft_packets::handshaking::handshake_packet::HandshakePacket;

    fn client(server_state: &ServerState, protocol_version: i32) -> ClientState {
        let mut client_state = ClientState::default();
        let handshake_packet = HandshakePacket::localhost(protocol_version, 1);
        handshake_packet
            .handle(&mut client_state, server_state)
            .unwrap();
        client_state
    }

    fn server() -> ServerState {
        let mut server_state_builder = ServerState::builder();
        server_state_builder.set_reply_to_status(true);
        server_state_builder.build().unwrap()
    }

    #[tokio::test]
    async fn test_should_respond_with_same_protocol_version_if_valid() {
        // Given
        let expected_protocol = 578;
        let server_state = server();
        let mut client_state = client(&server_state, expected_protocol);
        let status_request_packet = StatusRequestPacket::default();

        // When
        let batch = status_request_packet
            .handle(&mut client_state, &server_state)
            .unwrap();
        let mut batch = batch.into_stream();

        // Then
        let packet = batch.next().await.unwrap();
        assert!(matches!(
          packet,
          PacketRegistry::StatusResponse(ref status_packet)
           if status_packet.status_response().unwrap().version.protocol == expected_protocol
        ));
        assert!(batch.next().await.is_none());
    }

    #[tokio::test]
    async fn test_should_respond_with_any_version() {
        // Given
        let expected_protocol = -1;
        let server_state = server();
        let mut client_state = client(&server_state, expected_protocol);
        let status_request_packet = StatusRequestPacket::default();

        // When
        let batch = status_request_packet
            .handle(&mut client_state, &server_state)
            .unwrap();
        let mut batch = batch.into_stream();

        // Then
        let packet = batch.next().await.unwrap();
        assert!(matches!(
          packet,
          PacketRegistry::StatusResponse(ref status_packet)
           if status_packet.status_response().unwrap().version.protocol == expected_protocol
        ));
        assert!(batch.next().await.is_none());
    }

    #[tokio::test]
    async fn test_should_respond_with_latest_known_version_if_larger() {
        // Given
        let expected_protocol = i32::MAX;
        let server_state = server();
        let mut client_state = client(&server_state, expected_protocol);
        let status_request_packet = StatusRequestPacket::default();

        // When
        let batch = status_request_packet
            .handle(&mut client_state, &server_state)
            .unwrap();
        let mut batch = batch.into_stream();

        // Then
        let packet = batch.next().await.unwrap();
        assert!(matches!(
          packet,
          PacketRegistry::StatusResponse(ref status_packet)
           if status_packet.status_response().unwrap().version.protocol == ProtocolVersion::latest().version_number()
        ));
        assert!(batch.next().await.is_none());
    }

    #[tokio::test]
    async fn test_should_respond_with_oldest_known_version_if_smaller() {
        let test_values = [0, -2, i32::MIN];
        let server_state = server();

        for &expected_protocol in &test_values {
            // Given
            let mut client_state = client(&server_state, expected_protocol);
            let status_request_packet = StatusRequestPacket::default();

            // When
            let batch = status_request_packet
                .handle(&mut client_state, &server_state)
                .unwrap();
            let mut batch = batch.into_stream();

            // Then
            let packet = batch.next().await.unwrap();
            assert!(matches!(
                packet,
                PacketRegistry::StatusResponse(ref status_packet)
                    if status_packet.status_response().unwrap().version.protocol ==
                        ProtocolVersion::V1_7_2.version_number()
            ));
            assert!(batch.next().await.is_none());
        }
    }
}
