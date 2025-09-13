use crate::monitoring::metrics_provider::MetricsProvider;
use crate::server::client_data::ClientData;
use crate::server::packet_handler::{PacketHandler, PacketHandlerError};
use crate::server::packet_registry::{
    PacketRegistry, PacketRegistryDecodeError, PacketRegistryEncodeError,
};
use crate::server_state::ServerState;
use futures::StreamExt;
use minecraft_packets::login::login_disconnect_packet::LoginDisconnectPacket;
use minecraft_packets::play::client_bound_keep_alive_packet::ClientBoundKeepAlivePacket;
use minecraft_packets::play::disconnect_packet::DisconnectPacket;
use minecraft_protocol::prelude::State;
use net::packet_stream::PacketStreamError;
use net::raw_packet::RawPacket;
use std::io::ErrorKind;
use std::num::TryFromIntError;
use std::sync::Arc;
use thiserror::Error;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::RwLock;
use tracing::{debug, error, info, trace, warn};

pub struct Server {
    state: Arc<RwLock<ServerState>>,
    listen_address: String,
    metrics: Arc<dyn MetricsProvider>,
}

impl Server {
    pub fn new(
        listen_address: &impl ToString,
        state: ServerState,
        metrics: Arc<dyn MetricsProvider>,
    ) -> Self {
        Self {
            state: Arc::new(RwLock::new(state)),
            listen_address: listen_address.to_string(),
            metrics,
        }
    }

    pub async fn run(self) {
        let listener = match TcpListener::bind(&self.listen_address).await {
            Ok(sock) => sock,
            Err(err) => {
                error!("Failed to bind to {}: {}", self.listen_address, err);
                std::process::exit(1);
            }
        };

        info!("Listening on: {}", self.listen_address);
        self.accept(&listener).await;
    }

    pub async fn accept(self, listener: &TcpListener) {
        loop {
            let accept_result = listener.accept().await;
            match accept_result {
                Ok((socket, addr)) => {
                    debug!("Accepted connection from {}", addr);
                    self.metrics.inc_total_connections();
                    let state_clone = Arc::clone(&self.state);
                    let metrics_clone = Arc::clone(&self.metrics);
                    tokio::spawn(async move {
                        handle_client(socket, state_clone, metrics_clone).await;
                    });
                }
                Err(e) => {
                    error!("Failed to accept a connection: {:?}", e);
                }
            }
        }
    }
}

macro_rules! send_and_monitor_packet {
    ($client_data: expr, $metrics:expr, $protocol_version:expr, $packet:expr) => {{
        let raw_packet = $packet.encode_packet($protocol_version)?;
        $client_data.write_packet(raw_packet).await?;
        $metrics.inc_packets_sent($packet.packet_name(), $packet.state());
    }};
}

async fn handle_client(
    socket: TcpStream,
    server_state: Arc<RwLock<ServerState>>,
    metrics: Arc<dyn MetricsProvider>,
) {
    let client_data = ClientData::new(socket);
    let mut was_in_play_state = false;

    loop {
        match read(
            &client_data,
            &server_state,
            &mut was_in_play_state,
            &metrics,
        )
        .await
        {
            Ok(()) => {}
            Err(PacketProcessingError::PacketNotFound {
                version,
                state,
                packet_id,
            }) => {
                trace!(
                    "Unknown packet received: version={version} state={state} packet_id={packet_id}"
                );
                metrics.inc_packet_processing_error("PKT_NOT_FOUND");
            }
            Err(e) => {
                metrics.inc_packet_processing_error(e.error_code());
                break;
            }
        }
    }

    let _ = client_data.shutdown().await;

    if was_in_play_state {
        metrics.dec_connected_clients();
        server_state.write().await.decrement();
        let username = client_data.client().await.get_username();
        info!("{} left the game", username);
    }
}

async fn read(
    client_data: &ClientData,
    server_state: &Arc<RwLock<ServerState>>,
    was_in_play_state: &mut bool,
    metrics: &Arc<dyn MetricsProvider>,
) -> Result<(), PacketProcessingError> {
    tokio::select! {
        result = client_data.read_packet() => {
            let raw_packet = result?;
            process_packet(client_data, server_state, raw_packet, was_in_play_state, metrics).await?;
        }
        () = client_data.keep_alive_tick() => {
            send_keep_alive(client_data, metrics).await?;
        }
    }
    Ok(())
}

#[allow(clippy::cognitive_complexity)]
async fn process_packet(
    client_data: &ClientData,
    server_state: &Arc<RwLock<ServerState>>,
    raw_packet: RawPacket,
    was_in_play_state: &mut bool,
    metrics: &Arc<dyn MetricsProvider>,
) -> Result<(), PacketProcessingError> {
    let mut client_state = client_data.client().await;
    let protocol_version = client_state.protocol_version();
    let state = client_state.state();
    let decoded_packet = PacketRegistry::decode_packet(protocol_version, state, raw_packet)?;
    metrics.inc_packets_received(decoded_packet.packet_name(), decoded_packet.state());

    let batch = {
        let server_state_guard = server_state.read().await;
        decoded_packet.handle(&mut client_state, &server_state_guard)?
    };

    let protocol_version = client_state.protocol_version();
    let state = client_state.state();

    if !*was_in_play_state && state == State::Play {
        *was_in_play_state = true;
        server_state.write().await.increment();
        let username = client_state.get_username();
        debug!(
            "{} joined using version {}",
            username,
            protocol_version.humanize()
        );
        info!("{} joined the game", username);

        metrics.inc_connected_clients();
        metrics.inc_client_version(protocol_version.humanize());
    }

    let mut stream = batch.into_stream();
    while let Some(pending_packet) = stream.next().await {
        let enable_compression = matches!(pending_packet, PacketRegistry::SetCompression(..));
        send_and_monitor_packet!(client_data, metrics, protocol_version, pending_packet);
        if enable_compression
            && let Some(compression_settings) = server_state.read().await.compression_settings()
        {
            let mut packet_stream = client_data.stream().await;
            packet_stream
                .set_compression(compression_settings.threshold, compression_settings.level);
        }
    }

    if let Some(reason) = client_state.should_kick() {
        drop(client_state);
        kick_client(client_data, reason, metrics)
            .await
            .map_err(|_| PacketProcessingError::Disconnected)?;
        return Err(PacketProcessingError::Disconnected);
    }

    drop(client_state);
    client_data.enable_keep_alive_if_needed().await;

    Ok(())
}

async fn kick_client(
    client_data: &ClientData,
    reason: String,
    metrics: &Arc<dyn MetricsProvider>,
) -> Result<(), PacketProcessingError> {
    let (protocol_version, state) = {
        let state = client_data.client().await;
        (state.protocol_version(), state.state())
    };
    let packet = match state {
        State::Login => PacketRegistry::LoginDisconnect(LoginDisconnectPacket::text(reason)),
        State::Configuration => {
            PacketRegistry::ConfigurationDisconnect(DisconnectPacket::text(reason))
        }
        State::Play => PacketRegistry::PlayDisconnect(DisconnectPacket::text(reason)),
        _ => {
            warn!("A user was disconnected from a state where no packet can be sent");
            return Err(PacketProcessingError::Disconnected);
        }
    };

    send_and_monitor_packet!(client_data, metrics, protocol_version, packet);

    Ok(())
}

async fn send_keep_alive(
    client_data: &ClientData,
    metrics: &Arc<dyn MetricsProvider>,
) -> Result<(), PacketProcessingError> {
    let (protocol_version, state) = {
        let client = client_data.client().await;
        (client.protocol_version(), client.state())
    };

    if state == State::Play {
        let packet = PacketRegistry::ClientBoundKeepAlive(ClientBoundKeepAlivePacket::random()?);
        send_and_monitor_packet!(client_data, metrics, protocol_version, packet);
    }

    Ok(())
}

#[derive(Debug, Error)]
pub enum PacketProcessingError {
    #[error("Client disconnected")]
    Disconnected,

    #[error("Packet not found for version={version} state={state:?} packet_id=0x{packet_id:X}")]
    PacketNotFound {
        version: i32,
        state: State,
        packet_id: u8,
    },

    #[error("Packet handling failed")]
    Handler(#[from] PacketHandlerError),

    #[error("Packet encoding failed")]
    Encode(#[from] PacketRegistryEncodeError),

    #[error("Integer conversion failed")]
    IntConversion(#[from] TryFromIntError),

    #[error("Packet decoding failed")]
    Decode(#[source] PacketRegistryDecodeError),

    #[error("Packet stream I/O error")]
    Stream(#[source] PacketStreamError),
}

impl From<PacketStreamError> for PacketProcessingError {
    fn from(e: PacketStreamError) -> Self {
        if let PacketStreamError::Io(ref io_err) = e
            && (io_err.kind() == ErrorKind::UnexpectedEof
                || io_err.kind() == ErrorKind::ConnectionReset)
        {
            return Self::Disconnected;
        }
        Self::Stream(e)
    }
}

impl From<PacketRegistryDecodeError> for PacketProcessingError {
    fn from(e: PacketRegistryDecodeError) -> Self {
        match e {
            PacketRegistryDecodeError::NoCorrespondingPacket(version, state, packet_id) => {
                Self::PacketNotFound {
                    version,
                    state,
                    packet_id,
                }
            }
            _ => Self::Decode(e),
        }
    }
}

impl PacketProcessingError {
    pub const fn error_code(&self) -> &'static str {
        match self {
            Self::Disconnected => "NET_DISCONNECTED",
            Self::PacketNotFound { .. } => "PKT_NOT_FOUND",
            Self::Handler(_) => "PKT_HANDLER_ERR",
            Self::Encode(_) => "PKT_ENCODE_ERR",
            Self::IntConversion(_) => "ERR_INT_CONV",
            Self::Decode(_) => "PKT_DECODE_ERR",
            Self::Stream(_) => "NET_STREAM_ERR",
        }
    }
}
