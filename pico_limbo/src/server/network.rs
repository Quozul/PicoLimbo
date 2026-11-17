#![allow(
    clippy::too_many_lines,
    clippy::collapsible_if,
    clippy::significant_drop_in_scrutinee
)]
use crate::server::batch::{Batch, BatchItem};
use crate::server::client_data::ClientData;
use crate::server::packet_handler::{PacketHandler, PacketHandlerError};
use crate::server::packet_registry::{
    PacketRegistry, PacketRegistryDecodeError, PacketRegistryEncodeError,
};
use crate::server::server_address::ServerAddress;
use crate::server::shutdown_signal::shutdown_signal;
use crate::server_state::ServerState;
use futures::StreamExt;
use minecraft_packets::login::login_disconnect_packet::LoginDisconnectPacket;
use minecraft_packets::play::VoidChunkContext;
use minecraft_packets::play::block_update_packet::BlockUpdatePacket;
use minecraft_packets::play::chunk_data_and_update_light_packet::ChunkDataAndUpdateLightPacket;
use minecraft_packets::play::client_bound_keep_alive_packet::ClientBoundKeepAlivePacket;
use minecraft_packets::play::disconnect_packet::DisconnectPacket;
use minecraft_packets::play::set_action_bar_text_packet::SetActionBarTextPacket;
use minecraft_packets::play::set_title_text_packet::SetTitleTextPacket;
use minecraft_packets::play::synchronize_player_position_packet::SynchronizePlayerPositionPacket;
use minecraft_protocol::prelude::State;
use net::packet_stream::PacketStreamError;
use net::raw_packet::RawPacket;
use pico_text_component::prelude::parse_mini_message;
use std::num::TryFromIntError;
use std::sync::Arc;
use thiserror::Error;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, trace, warn};

pub struct Server {
    state: Arc<RwLock<ServerState>>,
    listen_address: ServerAddress,
}

impl Server {
    pub fn new(listen_address: &ServerAddress, state: ServerState) -> Self {
        Self {
            state: Arc::new(RwLock::new(state)),
            listen_address: listen_address.clone(),
        }
    }

    pub async fn run(
        self,
        cancellation_token: Option<&CancellationToken>,
    ) -> Result<(), std::io::Error> {
        let listener = TcpListener::bind(&self.listen_address.tuple()).await?;

        info!("Listening on: {}", self.listen_address);
        self.accept(&listener, cancellation_token).await;
        Ok(())
    }

    pub async fn accept(
        self,
        listener: &TcpListener,
        cancellation_token: Option<&CancellationToken>,
    ) {
        loop {
            tokio::select! {
                 accept_result = listener.accept() => {
                    match accept_result {
                        Ok((socket, addr)) => {
                            debug!("Accepted connection from {}", addr);
                        let state_clone = Arc::clone(&self.state);
                            tokio::spawn(async move {
                                handle_client(socket, state_clone).await;
                            });
                        }
                        Err(e) => {
                            error!("Failed to accept a connection: {:?}", e);
                        }
                    }
                },

                 () = shutdown_signal(cancellation_token) => {
                    info!("Shutdown signal received, shutting down gracefully.");
                    break;
                }
            }
        }
    }
}

#[derive(Debug, Error)]
pub enum PacketProcessingError {
    #[error("Client disconnected")]
    Disconnected,

    #[error("Packet not found version={0} state={1} packet_id={2}")]
    DecodePacketError(i32, State, u8),

    #[error("{0}")]
    Custom(String),
}

impl From<PacketHandlerError> for PacketProcessingError {
    fn from(e: PacketHandlerError) -> Self {
        match e {
            PacketHandlerError::Custom(reason) => Self::Custom(reason),
            PacketHandlerError::InvalidState(reason, should_warn) => {
                if should_warn {
                    warn!("{reason}");
                } else {
                    debug!("{reason}");
                }
                Self::Disconnected
            }
        }
    }
}

impl From<PacketRegistryDecodeError> for PacketProcessingError {
    fn from(e: PacketRegistryDecodeError) -> Self {
        match e {
            PacketRegistryDecodeError::NoCorrespondingPacket(version, state, packet_id) => {
                Self::DecodePacketError(version, state, packet_id)
            }
            _ => Self::Custom(e.to_string()),
        }
    }
}

impl From<PacketRegistryEncodeError> for PacketProcessingError {
    fn from(e: PacketRegistryEncodeError) -> Self {
        Self::Custom(e.to_string())
    }
}

impl From<TryFromIntError> for PacketProcessingError {
    fn from(e: TryFromIntError) -> Self {
        Self::Custom(e.to_string())
    }
}

impl From<PacketStreamError> for PacketProcessingError {
    fn from(value: PacketStreamError) -> Self {
        match value {
            PacketStreamError::Io(ref e)
                if e.kind() == std::io::ErrorKind::UnexpectedEof
                    || e.kind() == std::io::ErrorKind::ConnectionReset =>
            {
                Self::Disconnected
            }
            _ => Self::Custom(value.to_string()),
        }
    }
}

async fn process_packet(
    client_data: &ClientData,
    server_state: &Arc<RwLock<ServerState>>,
    raw_packet: RawPacket,
    was_in_play_state: &mut bool,
) -> Result<(), PacketProcessingError> {
    let (protocol_version, state) = {
        let client_state = client_data.client().await;
        (
            client_state.protocol_version(),
            client_state.serverbound_state(),
        )
    };
    let decoded_packet = PacketRegistry::decode_packet(protocol_version, state, raw_packet)?;
    let packet_name = decoded_packet.packet_name();
    trace!(packet_name, "Packet received");

    // Opt-in inbound packet hooks. Cancel skips handler (mini-game friendly).
    if state == State::Play {
        let server_state_guard = server_state.read().await;
        if server_state_guard.wants_inbound_packet_hooks() {
            let client_state = client_data.client().await;
            let username = client_state.get_username();
            let uuid = client_state
                .game_profile()
                .map(|profile| profile.uuid().to_string())
                .unwrap_or_default();
            drop(client_state);
            let event = format!("packet:{packet_name}");
            if server_state_guard.emit_player_plugin_event(&event, &username, &uuid) {
                return Ok(());
            }
        }
    }

    let mut batch: Batch = {
        let server_state_guard = server_state.read().await;
        let mut client_state = client_data.client().await;
        decoded_packet.handle(&mut client_state, &server_state_guard)?
    };

    if state == State::Play {
        let (username, spawn) = {
            let client_state = client_data.client().await;
            let server_state_guard = server_state.read().await;
            (
                client_state.get_username(),
                server_state_guard.spawn_position(),
            )
        };
        if !username.is_empty() {
            let actions = crate::plugins::PluginManager::take_client_actions(&username);
            for action in actions {
                match action {
                    crate::plugins::PluginClientAction::Title(text) => {
                        if let Ok(component) = parse_mini_message(&text) {
                            let packet = SetTitleTextPacket::new(&component);
                            batch.queue(|| PacketRegistry::SetTitleText(packet));
                        }
                    }
                    crate::plugins::PluginClientAction::ActionBar(text) => {
                        if let Ok(component) = parse_mini_message(&text) {
                            let packet = SetActionBarTextPacket::new(&component);
                            batch.queue(|| PacketRegistry::SetActionBarText(packet));
                        }
                    }
                    crate::plugins::PluginClientAction::TeleportSpawn => {
                        let (x, y, z) = spawn;
                        let packet = SynchronizePlayerPositionPacket::new(x, y, z, 0.0, 0.0);
                        batch.queue(|| PacketRegistry::SynchronizePlayerPosition(packet));
                    }
                    crate::plugins::PluginClientAction::BlockUpdate {
                        x,
                        y,
                        z,
                        internal_id,
                    } => {
                        if let Ok(mapping) =
                            blocks_report::get_block_report_id_mapping(protocol_version)
                        {
                            if let Some(report_id) =
                                blocks_report::get_block_id(&mapping, internal_id)
                            {
                                let packet = BlockUpdatePacket::new(x, y, z, i32::from(report_id));
                                batch.queue(|| PacketRegistry::BlockUpdate(packet));
                            }
                        }
                    }
                    crate::plugins::PluginClientAction::ResendChunk { chunk_x, chunk_z } => {
                        let dimension_height = 384;
                        let dimension_min_y = -64;
                        let context = VoidChunkContext {
                            chunk_x,
                            chunk_z,
                            biome_index: 0,
                            dimension_height,
                            dimension_min_y,
                            has_sky_light: true,
                            protocol_version,
                        };
                        let packet = ChunkDataAndUpdateLightPacket::void(context);
                        batch.queue(|| PacketRegistry::ChunkDataAndUpdateLight(Box::new(packet)));
                        // Re-apply sparse overlay blocks inside this chunk column.
                        for ((bx, by, bz), internal_id) in
                            crate::plugins::PluginManager::overlay_snapshot()
                        {
                            if bx >> 4 == chunk_x && bz >> 4 == chunk_z {
                                if let Ok(mapping) =
                                    blocks_report::get_block_report_id_mapping(protocol_version)
                                {
                                    if let Some(report_id) =
                                        blocks_report::get_block_id(&mapping, internal_id)
                                    {
                                        let packet = BlockUpdatePacket::new(
                                            bx,
                                            by,
                                            bz,
                                            i32::from(report_id),
                                        );
                                        batch.queue(|| PacketRegistry::BlockUpdate(packet));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    let (protocol_version, state) = {
        let client_state = client_data.client().await;
        (
            client_state.protocol_version(),
            client_state.serverbound_state(),
        )
    };

    if !*was_in_play_state && state == State::Play {
        *was_in_play_state = true;
        server_state.write().await.increment();
        let username = {
            let client_state = client_data.client().await;
            client_state.get_username()
        };
        debug!(
            "{} joined using version {}",
            username,
            protocol_version.humanize()
        );
        info!("{} joined the game", username);
        crate::plugins::PluginManager::note_player_online(&username);
    }

    let mut stream = batch.into_stream();
    while let Some(pending_packet) = stream.next().await {
        match pending_packet {
            BatchItem::Packet(packet) => {
                trace!(packet_name = packet.packet_name(), "Sending packet");
                let raw_packet = packet.encode_packet(protocol_version)?;
                client_data.write_packet(raw_packet).await?;
            }
            BatchItem::StateChange(direction, new_state) => {
                trace!(?direction, ?new_state, "Changing state");
                let mut client_state = client_data.client().await;
                client_state.set_state(direction, new_state);
            }
            BatchItem::EnableCompression => {
                if let Some(compression_settings) = server_state.read().await.compression_settings()
                {
                    let mut packet_stream = client_data.stream().await;
                    packet_stream.set_compression(
                        compression_settings.threshold,
                        compression_settings.level,
                    );
                } else {
                    warn!("compression enabled but no settings were found");
                }
            }
        }
    }

    {
        let mut client_state = client_data.client().await;
        let username = client_state.get_username();
        if !username.is_empty() {
            if let Some(reason) = server_state.read().await.take_plugin_kick(&username) {
                client_state.kick(&reason);
            }
        }
    }

    let should_kick = {
        let client_state = client_data.client().await;
        client_state.should_kick()
    };

    if let Some(reason) = should_kick {
        kick_client(client_data, reason.clone())
            .await
            .map_err(|_| PacketProcessingError::Disconnected)?;
        return Err(PacketProcessingError::Disconnected);
    }

    client_data.enable_keep_alive_if_needed().await;

    Ok(())
}

async fn read(
    client_data: &ClientData,
    server_state: &Arc<RwLock<ServerState>>,
    was_in_play_state: &mut bool,
) -> Result<(), PacketProcessingError> {
    tokio::select! {
        result = client_data.read_packet() => {
            let raw_packet = result?;
            process_packet(client_data, server_state, raw_packet, was_in_play_state).await?;
        }
        () = client_data.keep_alive_tick() => {
            server_state.read().await.tick_plugins();
            send_keep_alive(client_data).await?;
        }
    }
    Ok(())
}

async fn handle_client(socket: TcpStream, server_state: Arc<RwLock<ServerState>>) {
    let keep_alive_interval = server_state.read().await.keep_alive_interval();
    let client_data = ClientData::new(socket, keep_alive_interval);
    let mut was_in_play_state = false;

    loop {
        match read(&client_data, &server_state, &mut was_in_play_state).await {
            Ok(()) => {}
            Err(PacketProcessingError::Disconnected) => {
                debug!("Client disconnected");
                break;
            }
            Err(PacketProcessingError::Custom(e)) => {
                debug!("Error processing packet: {}", e);
            }
            Err(PacketProcessingError::DecodePacketError(version, state, packet_id)) => {
                trace!(
                    "Unknown packet received: version={version} state={state} packet_id={packet_id}"
                );
            }
        }
    }

    let _ = client_data.shutdown().await;

    if was_in_play_state {
        server_state.write().await.decrement();
        let client = client_data.client().await;
        let username = client.get_username();
        let uuid = client
            .game_profile()
            .map(|profile| profile.uuid().to_string())
            .unwrap_or_default();
        info!("{} left the game", username);
        crate::plugins::PluginManager::note_player_offline(&username);
        drop(client);
        server_state
            .read()
            .await
            .emit_player_plugin_event("player_quit", &username, &uuid);
    }
}

async fn kick_client(
    client_data: &ClientData,
    reason: String,
) -> Result<(), PacketProcessingError> {
    let (protocol_version, state) = {
        let state = client_data.client().await;
        (state.protocol_version(), state.clientbound_state())
    };
    let packet = match state {
        State::Login => {
            debug!("Login disconnect");
            PacketRegistry::LoginDisconnect(LoginDisconnectPacket::text(reason))
        }
        State::Configuration => {
            debug!("Configuration disconnect");
            PacketRegistry::ConfigurationDisconnect(DisconnectPacket::text(reason))
        }
        State::Play => {
            debug!("Play disconnect");
            PacketRegistry::PlayDisconnect(DisconnectPacket::text(reason))
        }
        _ => {
            debug!("A user was disconnected from a state where no packet can be sent");
            return Err(PacketProcessingError::Disconnected);
        }
    };
    if let Ok(raw_packet) = packet.encode_packet(protocol_version) {
        client_data.write_packet(raw_packet).await?;
        client_data.shutdown().await?;
    }

    Ok(())
}

async fn send_keep_alive(client_data: &ClientData) -> Result<(), PacketProcessingError> {
    let (protocol_version, state) = {
        let client = client_data.client().await;
        (client.protocol_version(), client.clientbound_state())
    };

    let packet = match state {
        State::Configuration => Some(PacketRegistry::ConfigurationClientBoundKeepAlive(
            ClientBoundKeepAlivePacket::random()?,
        )),
        State::Play => Some(PacketRegistry::ClientBoundKeepAlive(
            ClientBoundKeepAlivePacket::random()?,
        )),
        _ => None,
    };

    if let Some(packet) = packet {
        trace!(?state, "sending keep alive");
        let raw_packet = packet.encode_packet(protocol_version)?;
        client_data.write_packet(raw_packet).await?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn bind_failure_is_returned_to_the_caller() {
        let listener = TcpListener::bind(("127.0.0.1", 0))
            .await
            .expect("reserve a local port");
        let address = ServerAddress::parse(&listener.local_addr().unwrap().to_string())
            .expect("parse reserved address");
        let server = Server::new(&address, ServerState::default());

        assert!(server.run(None).await.is_err());
    }
}
