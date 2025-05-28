use crate::game_profile::GameProfile;
use crate::named_packet::NamedPacket;
use minecraft_packets::play::client_bound_keep_alive_packet::ClientBoundKeepAlivePacket;
use minecraft_protocol::data::packets_report::packet_map::PacketMap;
use minecraft_protocol::prelude::{EncodePacket, PacketId};
use minecraft_protocol::protocol_version::ProtocolVersion;
use minecraft_protocol::state::State;
use net::packet_stream::{PacketStream, PacketStreamError};
use net::raw_packet::RawPacket;
use rand::Rng;
use std::sync::Arc;
use thiserror::Error;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tracing::{debug, error};

pub struct Client {
    state: State,
    packet_reader: PacketStream<TcpStream>,
    packet_map: PacketMap,
    game_profile: Option<GameProfile>,
    version: Option<ProtocolVersion>,
    message_id: i32,
}

#[derive(Debug, Error)]
pub enum ClientReadPacketError {
    #[error(transparent)]
    PacketStream(#[from] PacketStreamError),
    #[error("unknown packet {id} received in state {state}")]
    UnknownPacket { id: u8, state: State },
    #[error("empty packet received in state {state}")]
    EmptyPacket { state: State },
}

impl Client {
    pub fn new(socket: TcpStream, packet_map: PacketMap) -> Self {
        let packet_reader = PacketStream::new(socket);
        Self {
            packet_reader,
            packet_map,
            state: State::default(),
            game_profile: None,
            version: None,
            message_id: -1,
        }
    }

    pub async fn read_packet(&mut self) -> Result<NamedPacket, ClientReadPacketError> {
        let packet = self.packet_reader.read_packet().await?;
        if let Some(packet_id) = packet.packet_id() {
            if let Some(packet_name) = self.get_packet_name_from_id(packet_id) {
                debug!("received packet {} (id={})", packet_name, packet_id);
                Ok(NamedPacket {
                    name: packet_name,
                    data: packet.data().to_vec(),
                })
            } else {
                Err(ClientReadPacketError::UnknownPacket {
                    id: packet_id,
                    state: self.state.clone(),
                })
            }
        } else {
            Err(ClientReadPacketError::EmptyPacket {
                state: self.state.clone(),
            })
        }
    }

    pub fn update_state(&mut self, new_state: State) {
        debug!("update state: {}", new_state);
        self.state = new_state;
    }

    pub async fn send_packet(&mut self, packet: impl EncodePacket + PacketId) {
        let version = self.version.clone().unwrap_or_default();
        let result: anyhow::Result<()> = async {
            let packet_id = self
                .packet_map
                .get_packet_id(&version, packet.get_packet_name())
                .ok()
                .flatten();

            if let Some(packet_id) = packet_id {
                debug!(
                    "sending packet {} (id={})",
                    packet.get_packet_name(),
                    packet_id
                );

                let raw_packet =
                    RawPacket::from_packet(packet_id, version.version_number(), &packet)?;
                self.packet_reader.write_packet(raw_packet).await?;
                Ok(())
            } else {
                error!(
                    "Trying to send an unmapped packet {}",
                    packet.get_packet_name()
                );
                Err(anyhow::anyhow!("No packet found"))
            }
        }
        .await;

        if let Err(err) = result {
            error!("error sending packet: {:?}", err);
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

    pub fn set_game_profile(&mut self, profile: GameProfile) {
        self.game_profile = Some(profile);
    }

    pub fn set_protocol(&mut self, protocol_version: ProtocolVersion) {
        debug!(
            "Client protocol version is {}",
            protocol_version.to_string()
        );
        self.version = Some(protocol_version);
    }

    pub fn protocol_version(&self) -> ProtocolVersion {
        self.version.clone().unwrap_or_default()
    }

    pub fn set_velocity_login_message_id(&mut self, message_id: i32) {
        self.message_id = message_id;
    }

    pub fn get_velocity_login_message_id(&self) -> i32 {
        self.message_id
    }

    fn get_packet_name_from_id(&self, packet_id: u8) -> Option<String> {
        self.packet_map
            .get_packet_name(&self.protocol_version(), &self.state, packet_id)
            .unwrap_or_else(|err| {
                error!("error getting packet name: {:?}", err);
                None
            })
    }
}

fn get_random() -> i64 {
    let mut rng = rand::rng();
    rng.random()
}

pub type SharedClient = Arc<Mutex<Client>>;
