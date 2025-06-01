use crate::client::Client;
use crate::client_inner::ClientReadPacketError;
use crate::connected_clients::ConnectedClients;
use crate::event_handler::{Handler, HandlerError, ListenerHandler};
use minecraft_protocol::data::packets_report::packet_map::PacketMap;
use minecraft_protocol::prelude::{DecodePacket, PacketId};
use minecraft_protocol::state::State;
use net::packet_stream::PacketStreamError;
use std::collections::HashMap;
use std::future::Future;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::signal;
use tokio::time::{Duration, interval};
use tracing::{debug, error, info};

pub struct Server<S>
where
    S: Clone + Sync + Send + ConnectedClients + 'static,
{
    state: S,
    handlers: HashMap<String, Box<dyn Handler<S>>>,
    listen_address: String,
    packet_map: PacketMap,
}

impl<S> Server<S>
where
    S: Clone + Sync + Send + ConnectedClients + 'static,
{
    pub fn new(listen_address: impl ToString, state: S, packet_map: PacketMap) -> Self {
        Self {
            state,
            packet_map,
            handlers: HashMap::new(),
            listen_address: listen_address.to_string(),
        }
    }

    pub fn on<T, F, Fut>(mut self, listener_fn: F) -> Self
    where
        T: PacketId + DecodePacket + Send + Sync + 'static,
        F: Fn(S, Client, T) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<(), HandlerError>> + Send + 'static,
    {
        let packet_name = T::PACKET_NAME.to_string();
        let handler = ListenerHandler::new(listener_fn);

        self.handlers.insert(packet_name, Box::new(handler));
        self
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

        let handlers = Arc::new(self.handlers);
        let packet_map = self.packet_map;

        loop {
            tokio::select! {
                accept_result = listener.accept() => {
                    match accept_result {
                        Ok((socket, addr)) => {
                            debug!("Accepted connection from {}", addr);
                            let handlers = handlers.clone();
                            let packet_map = packet_map.clone();
                            let state = self.state.clone();
                            tokio::spawn(async move {
                                handle_client(socket, handlers, packet_map, state.clone()).await;
                            });
                        }
                        Err(e) => {
                            error!("Failed to accept a connection: {:?}", e);
                        }
                    }
                },

                _ = signal::ctrl_c() => {
                    info!("SIGINT received, shutting down gracefully.");
                    break;
                }
            }
        }
    }
}

async fn handle_client<S: Clone + Sync + Send + ConnectedClients + 'static>(
    socket: TcpStream,
    handlers: Arc<HashMap<String, Box<dyn Handler<S>>>>,
    packet_map_clone: PacketMap,
    server_state: S,
) {
    let client = Client::new(socket, packet_map_clone);
    let mut keep_alive_interval = interval(Duration::from_secs(20));
    let mut was_in_play_state = false;

    loop {
        tokio::select! {
            packet_result = client.read_named_packet() => {
                match packet_result {
                    Ok(named_packet) => {
                        let packet_name_cache = named_packet.name.clone();
                        if let Some(handler) = handlers.get(&named_packet.name) {
                            if let Err(handler_error) = handler.handle(server_state.clone(), client.clone(), named_packet).await {
                                error!(
                                    "Handler for packet '{}' returned an error: {}",
                                    packet_name_cache,
                                    handler_error
                                );
                                break;
                            }

                            let current_client_state = client.current_state().await;
                            if current_client_state == State::Play && !was_in_play_state {
                                server_state.increment();
                                was_in_play_state = true;
                                let username = client.get_username().await;
                                info!("{} joined the game", username);
                            }
                        } else {
                            debug!("No handler for packet: {}", packet_name_cache);
                        }
                    }
                    Err(err) => {
                        match err {
                            ClientReadPacketError::UnknownPacketName { id, state, protocol, .. } => {
                                debug!("Unknown packet received 0x{:02X} in state {:?}, protocol {:?}", id, state, protocol);
                            }
                            ClientReadPacketError::PacketStream(PacketStreamError::IoError(ref io_err))
                                if io_err.kind() == tokio::io::ErrorKind::ConnectionReset ||
                                   io_err.kind() == tokio::io::ErrorKind::BrokenPipe ||
                                   io_err.kind() == tokio::io::ErrorKind::UnexpectedEof => {
                                debug!("Client disconnected cleanly: {:?}", err);
                                break;
                            }
                            ClientReadPacketError::PacketStream(PacketStreamError::IoError(io_err)) => {
                                error!("Client PacketStream IO error: {:?}", io_err);
                                break;
                            }
                            _ => {
                                error!("Error reading packet from client: {:?}", err);
                                break;
                            }
                        }
                    }
                }
            },

            _ = keep_alive_interval.tick() => {
                if client.current_state().await == State::Play {
                    if let Err(err) = client.send_keep_alive().await {
                        error!("Failed to send keep alive: {:?}", err);
                    }
                }
            },
        }
    }

    if was_in_play_state {
        server_state.decrement();
        let username = client.get_username().await;
        info!("{} left the game", username);
    } else {
        debug!("Client session ended (was not in play state).");
    }
}
