use crate::ping_util::parse_host_port::{parse_host_port, parse_socket_addr};
use minecraft_packets::handshaking::handshake_packet::HandshakePacket;
use minecraft_packets::status::status_request_packet::StatusRequestPacket;
use minecraft_packets::status::status_response_packet::StatusResponsePacket;
use minecraft_protocol::prelude::DecodePacket;
use minecraft_protocol::protocol_version::ProtocolVersion;
use net::packet_stream::PacketStream;
use net::raw_packet::RawPacket;
use std::process::ExitCode;
use std::str::FromStr;
use tokio::net::TcpStream;
use tracing::{error, info, warn};

pub async fn parse_cli_for_ping(address: String, json: bool, version: Option<String>) -> ExitCode {
    let protocol_version = version
        .map(|version_string| {
            ProtocolVersion::from_str(&version_string)
                .map_err(|_| warn!("Unknown version provided, fallback to latest version."))
                .unwrap_or_default()
        })
        .unwrap_or_default();

    let result = print_server_status(&address, protocol_version, json)
        .await
        .map_err(|err| {
            error!("{}", err);
        })
        .is_ok();

    if result {
        ExitCode::SUCCESS
    } else {
        error!("Could not ping the server.");
        ExitCode::FAILURE
    }
}

async fn print_server_status(
    server_address: &str,
    protocol_version: ProtocolVersion,
    json: bool,
) -> anyhow::Result<()> {
    let mut packet_reader = {
        let socket_addr = parse_socket_addr(server_address)?;
        let stream = TcpStream::connect(socket_addr).await?;
        PacketStream::<TcpStream>::new(stream)
    };

    {
        let (hostname, port) = parse_host_port(server_address);
        let handshake_packet = HandshakePacket::status(protocol_version, hostname, port);
        let packet =
            RawPacket::from_packet(0, protocol_version.version_number(), &handshake_packet)?;
        packet_reader.write_packet(packet).await?;
    }

    {
        let status_request = StatusRequestPacket::default();
        let packet = RawPacket::from_packet(0, protocol_version.version_number(), &status_request)?;
        packet_reader.write_packet(packet).await?;
    }

    let status_response = {
        let raw_packet = packet_reader.read_packet().await?;
        let status_response_packet =
            StatusResponsePacket::decode(raw_packet.data(), protocol_version.version_number())?;
        status_response_packet.status_response()?
    };

    if json {
        serde_json::to_writer_pretty(std::io::stdout(), &status_response)?;
    } else {
        info!(
            "Version: {} (protocol {})",
            status_response.version.name, status_response.version.protocol
        );

        let player_sample = match status_response.players.sample {
            Some(ref sample) if !sample.is_empty() => sample
                .iter()
                .map(|p| p.name.clone())
                .collect::<Vec<_>>()
                .join(", "),
            _ => "Player list is empty".to_string(),
        };

        info!(
            "Players ({}/{}): {}",
            status_response.players.online, status_response.players.max, player_sample
        );
    }

    Ok(())
}
