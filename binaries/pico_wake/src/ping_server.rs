use crate::handlers::handshake::GetStateProtocol;
use minecraft_packets::handshaking::handshake_packet::HandshakePacket;
use minecraft_packets::status::ping_request_packet::PingRequestPacket;
use minecraft_packets::status::ping_response_packet::PingResponsePacket;
use minecraft_protocol::prelude::DecodePacket;
use net::raw_packet::RawPacket;
use std::net::{SocketAddr, ToSocketAddrs};
use tokio::net::TcpStream;
use tracing::debug;

pub const HANDSHAKING_PACKET_ID: u8 = 0x00;
const PING_REQUEST_PACKET_ID: u8 = 0x01;

pub async fn ping_server(
    handshake_packet: &HandshakePacket,
    backend_server_address: &str,
) -> anyhow::Result<bool> {
    let addr = resolve_socket_addr(backend_server_address)?;
    debug!("server socket addr: {}", addr);
    let stream = TcpStream::connect(addr).await?;
    let mut packet_reader = net::packet_stream::PacketStream::new(stream);
    let protocol = handshake_packet.get_protocol();

    // Send handshake request
    {
        let new_handshake_packet = HandshakePacket {
            protocol: protocol.version_number().into(),
            hostname: addr.ip().to_string(),
            port: addr.port(),
            next_state: 1.into(), // We want to ping the server next
        };
        let raw_packet = RawPacket::from_packet(
            HANDSHAKING_PACKET_ID,
            protocol.version_number(),
            &new_handshake_packet,
        )?;
        packet_reader.write_packet(raw_packet).await?;
    }

    // Send ping request
    {
        let ping_request_packet = PingRequestPacket::default();
        let raw_packet = RawPacket::from_packet(
            PING_REQUEST_PACKET_ID,
            protocol.version_number(),
            &ping_request_packet,
        )?;
        packet_reader.write_packet(raw_packet).await?;

        // Wait for receiving ping response
        let packet = packet_reader.read_packet().await?;
        let ping_response = PingResponsePacket::decode(packet.data(), protocol.version_number())?;
        debug!(
            "Pinged backend server in {}ms",
            ping_request_packet.timestamp - ping_response.timestamp
        );
    }

    Ok(true)
}

fn resolve_socket_addr(addr: &str) -> std::io::Result<SocketAddr> {
    addr.to_socket_addrs()?
        .next()
        .ok_or_else(|| std::io::Error::other("no socket address found"))
}
