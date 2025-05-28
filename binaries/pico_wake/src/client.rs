use minecraft_packets::handshaking::handshake_packet::HandshakePacket;
use minecraft_protocol::data::packets_report::packet_map::PacketMap;
use minecraft_protocol::prelude::{EncodePacket, PacketId};
use minecraft_protocol::protocol_version::ProtocolVersion;
use minecraft_protocol::state::State;
use minecraft_server::named_packet::NamedPacket;
use net::packet_stream::{PacketStream, PacketStreamError};
use net::raw_packet::RawPacket;
use thiserror::Error;
use tokio::net::TcpStream;
use tracing::{debug, error};

pub struct Client {
    state: State,
    packet_reader: PacketStream<TcpStream>,
    packet_map: PacketMap,
    version: Option<ProtocolVersion>,
    backend_server_available: bool,
    handshake_packet_replay: Option<HandshakePacket>,
    wants_to_login: bool,
}

#[derive(Debug, Error)]
pub enum ClientReadPacketError {
    #[error(transparent)]
    PacketStream(#[from] PacketStreamError),
    #[error("unknown packet {id} received in state {state}")]
    UnknownPacket { id: u8, state: State },
    #[error("unknown packet received in state {state}")]
    EmptyPacket { state: State },
}

impl Client {
    pub fn new(socket: TcpStream, packet_map: PacketMap) -> Self {
        let packet_reader = PacketStream::new(socket);
        Self {
            packet_reader,
            packet_map,
            state: State::default(),
            version: None,
            backend_server_available: false,
            wants_to_login: false,
            handshake_packet_replay: None,
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

    pub fn set_backend_server_available(&mut self, packet: HandshakePacket) {
        self.backend_server_available = true;
        self.handshake_packet_replay = Some(packet);
    }

    pub fn set_wants_to_login(&mut self, proxy: bool) {
        self.wants_to_login = proxy;
    }

    pub fn wants_to_login(&self) -> bool {
        self.wants_to_login
    }

    pub fn get_handshake_packet_replay(&self) -> &Option<HandshakePacket> {
        &self.handshake_packet_replay
    }

    pub fn is_backend_server_available(&self) -> bool {
        self.backend_server_available
    }

    pub fn get_stream(&mut self) -> &mut TcpStream {
        self.packet_reader.get_stream()
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
