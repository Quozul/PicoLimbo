use crate::game_profile::GameProfile;
use crate::named_packet::NamedPacket;
use minecraft_protocol::data::packets_report::packet_map::PacketMap;
use minecraft_protocol::prelude::{EncodePacket, PacketId};
use minecraft_protocol::protocol_version::ProtocolVersion;
use minecraft_protocol::state::State;
use net::packet_stream::{PacketStream, PacketStreamError};
use net::raw_packet::RawPacket;
use thiserror::Error;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tracing::{debug, error};

pub struct ClientInner {
    state: State,
    packet_stream: PacketStream<TcpStream>,
    packet_map: PacketMap,
    game_profile: Option<GameProfile>,
    version: Option<ProtocolVersion>,
    message_id: i32,
}

impl ClientInner {
    pub fn new(socket: TcpStream, packet_map: PacketMap) -> Self {
        let packet_stream = PacketStream::new(socket);
        Self {
            packet_stream,
            packet_map,
            state: State::Handshake,
            game_profile: None,
            version: None,
            message_id: -1,
        }
    }

    fn get_packet_name_from_id_internal(
        &self,
        packet_id: u8,
        version: &ProtocolVersion,
    ) -> Result<String, ClientReadPacketError> {
        self.packet_map
            .get_packet_name(version, &self.state, packet_id)
            .map_err(|_| ClientReadPacketError::UnknownPacketId {
                id: packet_id,
                state: self.state.clone(),
            })
            .and_then(|opt_name| {
                opt_name.ok_or_else(|| ClientReadPacketError::UnknownPacketName {
                    name: format!("Unregistered ID 0x{:02X}", packet_id),
                    id: packet_id,
                    state: self.state.clone(),
                    protocol: version.clone(),
                })
            })
    }

    pub async fn read_named_packet_inner(&mut self) -> Result<NamedPacket, ClientReadPacketError> {
        let raw_packet = self.packet_stream.read_packet().await?;

        let current_version = self.version.clone().unwrap_or_else(|| {
            if self.state == State::Handshake {
                ProtocolVersion::default()
            } else {
                error!("CRITICAL: Protocol version not set while reading packet in state {:?}. This will likely lead to errors.", self.state);
                ProtocolVersion::default()
            }
        });

        if let Some(packet_id) = raw_packet.packet_id() {
            let packet_name = self.get_packet_name_from_id_internal(packet_id, &current_version)?;
            debug!("Received packet {} (id=0x{:02X})", packet_name, packet_id);
            Ok(NamedPacket {
                name: packet_name,
                data: raw_packet.data().to_vec(),
            })
        } else {
            Err(ClientReadPacketError::EmptyPacketData {
                state: self.state.clone(),
            })
        }
    }

    pub async fn send_encodable_packet_inner(
        &mut self,
        packet: impl EncodePacket + PacketId + Send,
    ) -> Result<(), ClientSendPacketError> {
        let version = self
            .version
            .clone()
            .ok_or_else(|| ClientSendPacketError::VersionNotSet {
                packet_name: packet.get_packet_name().to_string(),
            })?;

        let packet_name_str = packet.get_packet_name();

        let packet_id = self
            .packet_map
            .get_packet_id(&version, packet_name_str)
            .map_err(|e| {
                error!(
                    "Error looking up packet_id for {}: {:?}",
                    packet_name_str, e
                );
                ClientSendPacketError::UnmappedPacket {
                    packet_name: packet_name_str.to_owned(),
                    version: version.clone(),
                    state: self.state.clone(),
                }
            })?
            .ok_or_else(|| ClientSendPacketError::UnmappedPacket {
                packet_name: packet_name_str.to_owned(),
                version: version.clone(),
                state: self.state.clone(),
            })?;

        debug!(
            "Sending packet {} (id=0x{:02X})",
            packet_name_str, packet_id
        );

        let raw_packet = RawPacket::from_packet(packet_id, version.version_number(), &packet)
            .map_err(|_| ClientSendPacketError::EncodingError {
                packet_name: packet_name_str.to_owned(),
            })?;

        self.packet_stream.write_packet(raw_packet).await?;
        Ok(())
    }

    pub fn current_state(&self) -> &State {
        &self.state
    }

    pub fn set_state(&mut self, new_state: State) {
        debug!(
            "ClientInner state changing from {:?} to {:?}",
            self.state, new_state
        );
        self.state = new_state;
    }

    pub fn set_game_profile_inner(&mut self, profile: GameProfile) {
        self.game_profile = Some(profile);
    }

    pub fn game_profile_inner(&self) -> Option<&GameProfile> {
        self.game_profile.as_ref()
    }

    pub fn set_protocol_inner(&mut self, protocol_version: ProtocolVersion) {
        debug!(
            "ClientInner protocol version set to {}",
            protocol_version.to_string()
        );
        self.version = Some(protocol_version);
    }

    pub fn protocol_version_inner(&self) -> Option<ProtocolVersion> {
        self.version.clone()
    }

    pub fn set_velocity_login_message_id_inner(&mut self, message_id: i32) {
        self.message_id = message_id;
    }

    pub fn get_velocity_login_message_id_inner(&self) -> i32 {
        self.message_id
    }

    pub async fn shutdown(&mut self) -> std::io::Result<()> {
        self.packet_stream.get_stream().shutdown().await
    }
}

#[derive(Debug, Error)]
pub enum ClientReadPacketError {
    #[error(transparent)]
    PacketStream(#[from] PacketStreamError),
    #[error("unknown packet id=0x{id:02X} received in state {state}")]
    UnknownPacketId { id: u8, state: State },
    #[error(
        "unknown packet name '{name}' (id=0x{id:02X}) for state {state} & protocol {protocol:?}"
    )]
    UnknownPacketName {
        name: String,
        id: u8,
        state: State,
        protocol: ProtocolVersion,
    },
    #[error("empty packet data received in state {state}")]
    EmptyPacketData { state: State },
}

#[derive(Debug, Error)]
pub enum ClientSendPacketError {
    #[error(
        "packet '{packet_name}' not found in packet map for version {version:?} / state {state:?}"
    )]
    UnmappedPacket {
        packet_name: String,
        version: ProtocolVersion,
        state: State,
    },
    #[error("failed to encode packet '{packet_name}'")]
    EncodingError { packet_name: String },
    #[error(transparent)]
    PacketStream(#[from] PacketStreamError),
    #[error("client protocol version not set, cannot send packet '{packet_name}'")]
    VersionNotSet { packet_name: String },
    #[error(transparent)]
    Io(#[from] std::io::Error),
}
