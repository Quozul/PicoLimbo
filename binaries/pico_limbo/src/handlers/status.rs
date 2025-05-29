use crate::ServerState;
use minecraft_packets::status::data::status_response::StatusResponse;
use minecraft_packets::status::ping_request_packet::PingRequestPacket;
use minecraft_packets::status::ping_response_packet::PingResponsePacket;
use minecraft_packets::status::status_request_packet::StatusRequestPacket;
use minecraft_packets::status::status_response_packet::StatusResponsePacket;
use minecraft_server::client::SharedClient;

pub async fn on_status_request(
    state: ServerState,
    client: SharedClient,
    _packet: StatusRequestPacket,
) {
    let mut client = client.lock().await;
    let version = client.protocol_version();
    let status_response = StatusResponse::new(
        version.humanize(),
        version.version_number(),
        state.description_text(),
        state.max_players(),
        false,
    );
    let packet = StatusResponsePacket::from_status_response(&status_response);
    client.send_packet(packet).await;
}

pub async fn on_ping_request(_state: ServerState, client: SharedClient, packet: PingRequestPacket) {
    let packet = PingResponsePacket {
        timestamp: packet.timestamp,
    };
    client.lock().await.send_packet(packet).await;
}
