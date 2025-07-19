use crate::ServerState;
use crate::server::client::Client;
use crate::server::event_handler::HandlerError;
use minecraft_packets::status::data::status_response::StatusResponse;
use minecraft_packets::status::ping_request_packet::PingRequestPacket;
use minecraft_packets::status::ping_response_packet::PingResponsePacket;
use minecraft_packets::status::status_request_packet::StatusRequestPacket;
use minecraft_packets::status::status_response_packet::StatusResponsePacket;
use minecraft_protocol::protocol_version::ProtocolVersion;

pub async fn on_status_request(
    state: ServerState,
    client: Client,
    _packet: StatusRequestPacket,
) -> Result<(), HandlerError> {
    let (version_string, version_number) = if client.is_any_version().await {
        let oldest = ProtocolVersion::oldest();
        let latest = ProtocolVersion::latest();
        let version_string = format!("{oldest}-{latest}");
        (version_string, -1)
    } else {
        let protocol_version = client.protocol_version().await;
        (
            protocol_version.humanize().to_string(),
            protocol_version.version_number(),
        )
    };

    let status_response = StatusResponse::new(
        version_string,
        version_number,
        state.description_text(),
        state.online_players(),
        state.max_players(),
        false,
    );
    let packet = StatusResponsePacket::from_status_response(&status_response);
    client.send_packet(packet).await?;
    Ok(())
}

pub async fn on_ping_request(
    _state: ServerState,
    client: Client,
    packet: PingRequestPacket,
) -> Result<(), HandlerError> {
    let packet = PingResponsePacket {
        timestamp: packet.timestamp,
    };
    client.send_packet(packet).await?;
    Ok(())
}
