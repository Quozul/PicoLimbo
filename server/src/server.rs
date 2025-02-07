use crate::packet_reader::{PacketReaderError, PacketStream};
use crate::packets::play::client_bound_keep_alive_packet::ClientBoundKeepAlivePacket;
use crate::state::State;
use async_trait::async_trait;
use protocol::prelude::{DecodePacket, EncodePacket, PacketId};
use rand::Rng;
use std::collections::HashMap;
use std::future::Future;
use std::marker::PhantomData;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio::time::interval;
use tracing::{debug, error, info, warn};
// === Client and SharedClient definitions ===

pub type PacketMap = HashMap<(State, u8), String>;

pub struct Client {
    state: State,
    packet_reader: PacketStream,
    packet_map: PacketMap,
}

impl Client {
    pub fn new(socket: TcpStream, packet_map: PacketMap) -> Self {
        let packet_reader = PacketStream::new(socket);
        Self {
            packet_reader,
            packet_map,
            state: State::default(),
        }
    }

    pub async fn read_packet(&mut self) -> Result<Option<RawPacket>, PacketReaderError> {
        self.packet_reader.read_packet().await.map(|packet| {
            self.get_packet_name_from_id(packet.id())
                .map(|packet_name| RawPacket {
                    name: packet_name,
                    data: packet.data().to_vec(),
                })
        })
    }

    pub fn update_state(&mut self, new_state: State) {
        debug!("update state: {}", new_state);
        self.state = new_state;
    }

    pub async fn send_packet(&mut self, packet: impl EncodePacket + PacketId) {
        if let Err(err) = self.packet_reader.write_packet(packet).await {
            error!("failed to send packet: {}", err);
        }
    }

    pub fn state(&self) -> &State {
        &self.state
    }

    pub async fn send_keep_alive(&mut self) {
        // Send Keep Alive
        if self.state() == &State::Play {
            let packet = ClientBoundKeepAlivePacket::new(get_random());
            self.send_packet(packet).await;
        }
    }

    fn get_packet_name_from_id(&self, packet_id: u8) -> Option<String> {
        self.packet_map
            .get(&(self.state.clone(), packet_id))
            .map(|s| s.to_string())
    }
}

fn get_random() -> i64 {
    let mut rng = rand::rng();
    rng.random()
}

pub type SharedClient = Arc<Mutex<Client>>;

// === Packet decoding and RawPacket ===

fn decode_packet<T: DecodePacket>(raw_packet: RawPacket) -> T {
    T::decode(&raw_packet.data).unwrap()
}

pub struct RawPacket {
    name: String,
    data: Vec<u8>,
}

// === Server and its methods ===

pub struct Server {
    handlers: HashMap<String, Box<dyn Handler>>,
    listen_address: String,
    packet_map: PacketMap,
}

impl Server {
    pub fn new(listen_address: impl ToString) -> Self {
        Self {
            handlers: HashMap::new(),
            packet_map: HashMap::new(),
            listen_address: listen_address.to_string(),
        }
    }

    pub fn on<T, F, Fut>(mut self, state: State, listener_fn: F) -> Self
    where
        T: PacketId + DecodePacket + Send + Sync + 'static,
        F: Fn(SharedClient, T) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let packet_name = T::PACKET_NAME.to_string();
        let handler = ListenerHandler::<T, F> {
            listener_fn: Arc::new(listener_fn),
            _marker: PhantomData,
        };

        self.packet_map
            .insert((state, T::PACKET_ID), packet_name.clone());
        self.handlers.insert(packet_name, Box::new(handler));
        self
    }

    pub async fn run(self) {
        let listener = TcpListener::bind(&self.listen_address)
            .await
            .expect("Failed to bind address");
        info!("listening on: {}", self.listen_address);

        let handlers = Arc::new(self.handlers);
        let packet_map = self.packet_map;

        loop {
            match listener.accept().await {
                Ok((socket, addr)) => {
                    info!("Accepted connection from {}", addr);
                    let handlers = handlers.clone();
                    let packet_map = packet_map.clone();
                    tokio::spawn(async move {
                        if let Err(e) = handle_client(socket, handlers, packet_map).await {
                            error!("Error handling client {}: {:?}", addr, e);
                        }
                    });
                }
                Err(e) => {
                    error!("Failed to accept a connection: {:?}", e);
                }
            }
        }
    }
}

async fn handle_client(
    socket: TcpStream,
    handlers: Arc<HashMap<String, Box<dyn Handler>>>,
    packet_map: PacketMap,
) -> tokio::io::Result<()> {
    let client = Arc::new(Mutex::new(Client::new(socket, packet_map)));
    let mut keep_alive_interval = interval(Duration::from_secs(20));

    loop {
        tokio::select! {
            packet_result = async {
                client.lock().await.read_packet().await
            } => {
                match packet_result {
                    Ok(Some(raw_packet)) => {
                        if let Some(handler) = handlers.get(&raw_packet.name) {
                            handler.handle(client.clone(), raw_packet).await;
                        } else {
                            error!("No handler registered for packet: {}", raw_packet.name);
                        }
                    }
                    Ok(None) => {
                        // You can decide what to do if no packet was received.
                    }
                    Err(err) => {
                        warn!("Client disconnected or error reading packet: {:?}", err);
                        break;
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

// === Handler trait and ListenerHandler ===

#[async_trait]
pub trait Handler: Send + Sync {
    async fn handle(&self, client: SharedClient, raw_packet: RawPacket);
}

struct ListenerHandler<T, F> {
    listener_fn: Arc<F>,
    _marker: PhantomData<T>,
}

#[async_trait]
impl<T, F, Fut> Handler for ListenerHandler<T, F>
where
    T: DecodePacket + Send + Sync + 'static,
    F: Fn(SharedClient, T) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = ()> + Send + 'static,
{
    async fn handle(&self, client: SharedClient, raw_packet: RawPacket) {
        let packet = decode_packet::<T>(raw_packet);
        (self.listener_fn)(client, packet).await;
    }
}
