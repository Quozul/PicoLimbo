use crate::server::client_state::ClientState;
use crate::server::packet_handler::{PacketHandler, PacketHandlerError};
use crate::server::packet_registry::PacketRegistry;
use crate::server_state::ServerState;
use minecraft_packets::status::data::status_response::StatusResponse;
use minecraft_packets::status::ping_request_packet::PingRequestPacket;
use minecraft_packets::status::ping_response_packet::PongResponsePacket;
use minecraft_packets::status::status_request_packet::StatusRequestPacket;
use minecraft_packets::status::status_response_packet::StatusResponsePacket;
use minecraft_protocol::prelude::ProtocolVersion;

impl PacketHandler for StatusRequestPacket {
    fn handle(
        &self,
        client_state: &mut ClientState,
        server_state: &ServerState,
    ) -> Result<(), PacketHandlerError> {
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
            server_state.description_text(),
            server_state.online_players(),
            server_state.max_players(),
            false,
        );
        let packet = StatusResponsePacket::from_status_response(&status_response);
        client_state.queue_packet(PacketRegistry::StatusResponse(packet))?;
        Ok(())
    }
}

impl PacketHandler for PingRequestPacket {
    fn handle(
        &self,
        client_state: &mut ClientState,
        _server_state: &ServerState,
    ) -> Result<(), PacketHandlerError> {
        let packet = PongResponsePacket {
            timestamp: self.timestamp,
        };
        client_state.queue_packet(PacketRegistry::PongResponse(packet))?;
        Ok(())
    }
}
