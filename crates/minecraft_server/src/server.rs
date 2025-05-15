use crate::client::{Client, ClientReadPacketError, SharedClient};
use crate::event_handler::{Handler, ListenerHandler};
use minecraft_protocol::data::packets_report::packet_map::PacketMap;
use minecraft_protocol::prelude::{DecodePacket, PacketId};
use std::collections::HashMap;
use std::future::Future;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::{TcpListener, TcpStream};
use tokio::signal;
use tokio::sync::Mutex;
use tokio::time::interval;
use tracing::{debug, error, info};

pub struct Server<S>
where
    S: Clone + Sync + Send + 'static,
{
    state: S,
    handlers: HashMap<String, Box<dyn Handler<S>>>,
    listen_address: String,
    packet_map: PacketMap,
}

impl<S: Clone + Sync + Send + 'static> Server<S> {
    pub fn new(listen_address: impl ToString, data_location: PathBuf, state: S) -> Self {
        Self {
            state,
            handlers: HashMap::new(),
            packet_map: PacketMap::new(data_location),
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
                                if let Err(e) = handle_client(socket, handlers, packet_map, state).await {
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

async fn handle_client<S: Clone>(
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

    Ok(())
}
