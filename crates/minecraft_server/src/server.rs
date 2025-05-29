use crate::client::{Client, ClientReadPacketError, SharedClient};
use crate::event_handler::{Handler, ListenerHandler};
use minecraft_packets::play::Dimension;
use minecraft_protocol::data::packets_report::packet_map::PacketMap;
use minecraft_protocol::prelude::{DecodePacket, PacketId};
use minecraft_protocol::state::State;
use std::collections::HashMap;
use std::future::Future;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use tokio::net::{TcpListener, TcpStream};
use tokio::signal;
use tokio::sync::Mutex;
use tokio::time::{Duration, interval};
use tracing::{debug, error, info};

pub trait GetDataDirectory {
    fn data_directory(&self) -> &PathBuf;

    fn spawn_dimension(&self) -> &Dimension;

    fn connected_clients(&self) -> &Arc<AtomicU32>;
}

pub struct Server<S>
where
    S: Clone + Sync + Send + GetDataDirectory + 'static,
{
    state: S,
    handlers: HashMap<String, Box<dyn Handler<S>>>,
    listen_address: String,
    packet_map: PacketMap,
}

impl<S> Server<S>
where
    S: Clone + Sync + Send + GetDataDirectory + 'static,
{
    pub fn new(listen_address: impl ToString, state: S) -> Self {
        let asset_directory = state.data_directory().clone();
        Self {
            state,
            handlers: HashMap::new(),
            packet_map: PacketMap::new(asset_directory),
            listen_address: listen_address.to_string(),
        }
    }

    pub fn on<T, F, Fut>(mut self, listener_fn: F) -> Self
    where
        T: PacketId + DecodePacket + Send + Sync + 'static,
        F: Fn(S, SharedClient, T) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let packet_name = T::PACKET_NAME.to_string();
        let handler = ListenerHandler::new(listener_fn);

        self.handlers.insert(packet_name, Box::new(handler));
        self
    }

    pub async fn run(self) {
        let listener = TcpListener::bind(&self.listen_address)
            .await
            .expect("Failed to bind address");
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
                                if let Err(e) = handle_client(socket, handlers, packet_map, state.clone()).await {
                                    error!("Error handling client {}: {:?}", addr, e);
                                }
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

async fn handle_client<S: Clone + GetDataDirectory>(
    socket: TcpStream,
    handlers: Arc<HashMap<String, Box<dyn Handler<S>>>>,
    packet_map: PacketMap,
    state: S,
) -> tokio::io::Result<()> {
    let client = Arc::new(Mutex::new(Client::new(socket, packet_map)));
    let mut keep_alive_interval = interval(Duration::from_secs(20));

    loop {
        tokio::select! {
            packet_result = async {
                client.lock().await.read_packet().await
            } => {
                match packet_result {
                    Ok(named_packet) => {
                        if let Some(handler) = handlers.get(&named_packet.name) {
                            handler.handle(state.clone(), client.clone(), named_packet).await;

                            if client.lock().await.state() == &State::Play {
                                state.connected_clients().fetch_add(1, Ordering::SeqCst);
                            }
                        }
                        // Silently ignore no handler
                    }
                    Err(err) => {
                        match err {
                            ClientReadPacketError::PacketStream(err) => {
                                debug!("client disconnected or error reading packet: {:?}", err);
                                break;
                            }
                            err => {
                                debug!("{err}");
                            }
                        }

                    }
                }
            },

            _ = keep_alive_interval.tick() => {
                client.lock().await.send_keep_alive().await;
            },
        }
    }

    if client.lock().await.state() == &State::Play {
        state.connected_clients().fetch_sub(1, Ordering::SeqCst);
    }

    Ok(())
}
