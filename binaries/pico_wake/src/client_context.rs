use crate::client::Client;
use crate::connection_handler::ConnectionHandler;
use crate::handlers::handshake::on_handshake;
use crate::handlers::login::on_login_start;
use crate::handlers::status::{on_ping_request, on_status_request};
use crate::ping_server::HANDSHAKING_PACKET_ID;
use crate::server_manager::{ServerManager, ServerStatus};
use async_trait::async_trait;
use minecraft_packets::handshaking::handshake_packet::HandshakePacket;
use minecraft_packets::login::login_state_packet::LoginStartPacket;
use minecraft_packets::status::ping_request_packet::PingRequestPacket;
use minecraft_packets::status::status_request_packet::StatusRequestPacket;
use minecraft_protocol::data::packets_report::packet_map::PacketMap;
use minecraft_protocol::prelude::PacketId;
use net::raw_packet::RawPacket;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;
use tokio::io::copy_bidirectional;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tracing::{debug, error, info};

#[derive(Clone)]
pub struct ClientContext {
    pub packet_map: PacketMap,
    pub backend_server_address: String,
    pub server_manager: Arc<Mutex<ServerManager>>,
    pub connected_players: Arc<AtomicUsize>,
    pub sleep_delay: Duration,
}

impl ClientContext {
    /// Constructs a new `ClientContext` by reading configuration from environment variables.
    ///
    /// Expected environment variables:
    /// - `STARTUP`: Required. Used for creating the `ServerManager`.
    pub fn new(backend_server_address: String) -> anyhow::Result<Self> {
        let packet_map = PacketMap::new(PathBuf::from("./assets"));

        // Create the server manager using the STARTUP environment variable.
        let startup = std::env::var("STARTUP")?;
        let server_manager = ServerManager::new(startup);
        let server_manager = Arc::new(Mutex::new(server_manager));

        // Initialize the connected players counter.
        let connected_players = Arc::new(AtomicUsize::new(0));

        // Initialize the sleep delay.
        let sleep_delay = Duration::from_secs(
            std::env::var("SLEEP_DELAY")
                .unwrap_or_else(|_| "3600".to_string())
                .parse::<u64>()
                .unwrap_or(3600),
        );

        Ok(ClientContext {
            packet_map,
            backend_server_address,
            server_manager,
            connected_players,
            sleep_delay,
        })
    }
}

#[async_trait]
impl ConnectionHandler for ClientContext {
    async fn on_accept(&self, tcp_stream: TcpStream, _addr: SocketAddr) -> anyhow::Result<()> {
        let mut client = Client::new(tcp_stream, self.packet_map.clone());
        loop {
            if let Err(err) = handle_client(
                &mut client,
                &self.backend_server_address,
                &self.server_manager,
            )
            .await
            {
                error!("Failed to handle client: {err}");
                break;
            }

            if client.is_backend_server_available() {
                self.connected_players.fetch_add(1, Ordering::SeqCst);
                if client.wants_to_login() {
                    let server_manager = self.server_manager.lock().await;
                    server_manager.cancel_stop().await;
                }

                if let Err(err) = start_proxying(&mut client, &self.backend_server_address).await {
                    error!("Failed while proxying: {err}");
                }
                self.connected_players.fetch_sub(1, Ordering::SeqCst);
                break;
            }
        }

        // On client disconnect, check if we can stop the server
        let player_count = self.connected_players.load(Ordering::SeqCst);
        if player_count == 0 {
            let server_manager = self.server_manager.lock().await;
            if server_manager.get_server_status().await != ServerStatus::Offline {
                if client.wants_to_login() {
                    server_manager.cancel_stop().await;
                }

                server_manager.schedule_stop(self.sleep_delay).await;
            }
        }

        Ok(())
    }

    async fn on_stop(&self) -> anyhow::Result<()> {
        info!("Stopping server");
        let server_manager = self.server_manager.lock().await;
        server_manager.cancel_stop().await;
        server_manager.schedule_stop(Duration::ZERO).await;
        server_manager.stop_stdin_listener();
        Ok(())
    }
}

async fn handle_client(
    client: &mut Client,
    backend_server_address: &str,
    server_manager: &Arc<Mutex<ServerManager>>,
) -> anyhow::Result<()> {
    let packet = client.read_packet().await?;
    match packet.name.as_ref() {
        HandshakePacket::PACKET_NAME => {
            on_handshake(
                client,
                &packet.decode::<HandshakePacket>(client.protocol_version())?,
                backend_server_address,
                server_manager,
            )
            .await
        }
        LoginStartPacket::PACKET_NAME => {
            on_login_start(
                client,
                &packet.decode::<LoginStartPacket>(client.protocol_version())?,
            )
            .await
        }
        StatusRequestPacket::PACKET_NAME => {
            on_status_request(
                client,
                &packet.decode::<StatusRequestPacket>(client.protocol_version())?,
                server_manager,
            )
            .await
        }
        PingRequestPacket::PACKET_NAME => {
            on_ping_request(
                client,
                &packet.decode::<PingRequestPacket>(client.protocol_version())?,
            )
            .await
        }
        _ => {}
    }
    Ok(())
}

async fn start_proxying(client: &mut Client, backend_server_address: &str) -> anyhow::Result<()> {
    debug!("Starting proxying to {}", backend_server_address);
    let mut outbound = TcpStream::connect(backend_server_address).await?;

    {
        // Replay handshake packet of the client
        if let Some(handshake_packet) = client.get_handshake_packet_replay() {
            let raw_packet = RawPacket::from_packet(
                HANDSHAKING_PACKET_ID,
                client.protocol_version().version_number(),
                handshake_packet,
            )?;
            let mut packet_stream = net::packet_stream::PacketStream::new(&mut outbound);
            packet_stream.write_packet(raw_packet).await?;
        }
    }

    let stream = client.get_stream();
    copy_bidirectional(stream, &mut outbound).await?;
    Ok(())
}
