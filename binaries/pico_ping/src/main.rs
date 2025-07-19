mod cli;
mod parse_host_port;

use crate::cli::Cli;
use crate::parse_host_port::{parse_host_port, parse_socket_addr};
use clap::Parser;
use minecraft_packets::handshaking::handshake_packet::HandshakePacket;
use minecraft_packets::status::status_request_packet::StatusRequestPacket;
use minecraft_packets::status::status_response_packet::StatusResponsePacket;
use minecraft_protocol::prelude::DecodePacket;
use minecraft_protocol::protocol_version::ProtocolVersion;
use net::packet_stream::PacketStream;
use net::raw_packet::RawPacket;
use std::str::FromStr;
use tokio::net::TcpStream;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let server_address = &cli.address;

    let protocol_version = if let Some(version) = cli.version {
        ProtocolVersion::from_str(&version)?
    } else {
        ProtocolVersion::latest()
    };

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

    if cli.json {
        serde_json::to_writer_pretty(std::io::stdout(), &status_response)?;
    } else {
        println!(
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

        println!(
            "Players ({}/{}): {}",
            status_response.players.online, status_response.players.max, player_sample
        );
    }

    Ok(())
}
