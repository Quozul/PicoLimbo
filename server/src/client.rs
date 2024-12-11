use crate::packets::configuration::client_bound_known_packs_packet::ClientBoundKnownPacksPacket;
use crate::packets::configuration::client_bound_plugin_message_packet::ClientBoundPluginMessagePacket;
use crate::packets::configuration::data::registry_entry::RegistryEntry;
use crate::packets::configuration::data::server_link_label::ServerLinkLabel;
use crate::packets::configuration::finish_configuration_packet::FinishConfigurationPacket;
use crate::packets::configuration::registry_data_packet::RegistryDataPacket;
use crate::packets::configuration::server_links_packet::{ServerLink, ServerLinksPacket};
use crate::packets::login::login_success_packet::LoginSuccessPacket;
use crate::packets::play::chunk_data_and_update_light_packet::ChunkDataAndUpdateLightPacket;
use crate::packets::play::client_bound_keep_alive_packet::ClientBoundKeepAlivePacket;
use crate::packets::play::game_event_packet::GameEventPacket;
use crate::packets::play::login_packet::LoginPacket;
use crate::packets::play::synchronize_player_position_packet::SynchronizePlayerPositionPacket;
use crate::packets::status::ping_response_packet::PingResponsePacket;
use crate::packets::status::status_response::StatusResponse;
use crate::packets::status::status_response_packet::StatusResponsePacket;
use crate::payload::{Payload, PayloadAppendError};
use crate::registry::get_all_registries::get_all_registries;
use crate::state::handle_configuration_state::{handle_configuration_state, ConfigurationResult};
use crate::state::handle_handshake_state::handle_handshake_state;
use crate::state::handle_login_state::{handle_login_state, LoginResult};
use crate::state::handle_play_state::{handle_play_state, PlayResult};
use crate::state::handle_status_state::{handle_status_state, StatusResult};
use crate::state::State;
use protocol::prelude::{EncodePacket, Identifier, PacketId, SerializePacketData, VarInt};
use rand::Rng;
use std::collections::HashSet;
use std::fmt::Debug;
use std::path::Path;
use std::str::FromStr;
use thiserror::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tracing::{debug, error, warn};

pub struct Client {
    socket: TcpStream,
    state: State,
    payload: Payload,
}

#[derive(Error, Debug)]
pub enum ClientReadError {
    #[error("invalid packet_in received; error={0}")]
    InvalidPacket(PayloadAppendError),
    #[error("connection closed; error={0}")]
    ConnectionClosed(std::io::Error),
    #[error("failed to read socket; error={0}")]
    FailedToRead(std::io::Error),
    #[error("state not supported {0}")]
    NotSupportedState(State),
}

impl Client {
    pub fn new(socket: TcpStream) -> Client {
        Client {
            socket,
            state: State::Handshake,
            payload: Payload::new(),
        }
    }

    pub fn update_state(&mut self, new_state: State) {
        self.state = new_state;
        debug!("client state updated to {:?}", self.state);
    }

    pub async fn read_socket(&mut self) -> Result<(), ClientReadError> {
        let mut buf = vec![0; self.payload.get_remaining_to_read()];

        let bytes_received = self
            .socket
            .read(&mut buf)
            .await
            .map_err(ClientReadError::FailedToRead)?;

        if bytes_received == 0 {
            // Test if the socket is still open
            if let Err(err) = self.socket.write_all(&[0]).await {
                return Err(ClientReadError::ConnectionClosed(err));
            }
        }

        if let Err(err) = self
            .payload
            .append_bytes(&buf[..bytes_received], bytes_received)
        {
            return Err(ClientReadError::InvalidPacket(err));
        }

        Ok(())
    }

    pub fn is_complete(&self) -> bool {
        self.payload.is_complete()
    }

    pub fn get_payload(&self) -> &Payload {
        &self.payload
    }

    pub fn reset_payload(&mut self) -> Result<(), PayloadAppendError> {
        self.payload.reset()
    }

    pub async fn handle(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        match self.state {
            State::Handshake => self.handle_handshake().await,
            State::Status => self.handle_status().await,
            State::Login => self.handle_login().await,
            State::Configuration => self.handle_configuration().await,
            State::Play => self.handle_play().await,
            State::Transfer => Err(Box::new(ClientReadError::NotSupportedState(
                State::Transfer,
            ))),
        }
    }

    pub fn get_packet(&mut self) -> (u8, &[u8]) {
        let bytes = self.get_payload().get_data();
        let packet_id = bytes[0];
        let packet_payload = &bytes[1..];
        (packet_id, packet_payload)
    }

    async fn handle_play(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let (packet_id, packet_payload) = self.get_packet();
        let result = handle_play_state(packet_id, packet_payload);
        match result {
            Ok(result) => match result {
                PlayResult::UpdatePositionAndRotation { .. } => {}
                PlayResult::Nothing => {}
            },
            Err(err) => {
                warn!("{err}");
            }
        }
        Ok(())
    }

    async fn handle_configuration(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let (packet_id, packet_payload) = self.get_packet();
        let result = handle_configuration_state(packet_id, packet_payload)?;
        match result {
            ConfigurationResult::SendConfiguration => {
                // Send Server Brand
                let packet =
                    ClientBoundPluginMessagePacket::brand("Quozul's Custom Server Software");
                self.write_packet(packet).await?;

                // Send Known Packs
                let packet = ClientBoundKnownPacksPacket::default();
                self.write_packet(packet).await?;

                // Send Registry Data
                self.send_registry_data().await?;

                // Send Server Links
                let packet = ServerLinksPacket {
                    links: vec![ServerLink::built_in(
                        ServerLinkLabel::Website,
                        "https://quozul.dev",
                    )]
                    .into(),
                };
                self.write_packet(packet).await?;

                // Send Finished Configuration
                let packet = FinishConfigurationPacket {};
                self.write_packet(packet).await?;
                Ok(())
            }
            ConfigurationResult::Play => {
                self.update_state(State::Play);

                let packet = LoginPacket::default();
                self.write_packet(packet).await?;

                // Send Synchronize Player Position
                let packet = SynchronizePlayerPositionPacket::default();
                self.write_packet(packet).await?;

                // Send Game Event
                let packet = GameEventPacket::start_waiting_for_chunks(0.0);
                self.write_packet(packet).await?;

                // Send Chunk Data and Update Light
                let packet = ChunkDataAndUpdateLightPacket::default();
                self.write_packet(packet).await?;

                // Send Keep Alive
                self.send_keep_alive().await?;
                Ok(())
            }
            ConfigurationResult::Nothing => Ok(()),
        }
    }

    async fn send_registry_data(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let registries = get_all_registries(Path::new("./data/1_21_4/minecraft"));
        let registry_names = registries
            .iter()
            .map(|registry| registry.registry_id.clone())
            .collect::<HashSet<String>>();
        for registry_name in registry_names {
            let packet = RegistryDataPacket {
                registry_id: Identifier::from_str(&registry_name)?,
                entries: registries
                    .iter()
                    .filter(|entry| entry.registry_id == registry_name)
                    .map(|entry| RegistryEntry {
                        entry_id: Identifier::minecraft(&entry.entry_id),
                        has_data: true,
                        nbt: Some(entry.nbt.clone()),
                    })
                    .collect::<Vec<_>>()
                    .into(),
            };
            self.write_packet(packet).await?;
        }
        Ok(())
    }

    async fn handle_login(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let (packet_id, packet_payload) = self.get_packet();
        let result = handle_login_state(packet_id, packet_payload)?;
        match result {
            LoginResult::Login(uuid, username) => {
                let packet = LoginSuccessPacket {
                    uuid,
                    username,
                    properties: Vec::new().into(),
                };
                self.write_packet(packet).await?;
            }
            LoginResult::LoginAcknowledged => {
                self.update_state(State::Configuration);
            }
        }
        Ok(())
    }

    async fn handle_status(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let (packet_id, packet_payload) = self.get_packet();
        let result = handle_status_state(packet_id, packet_payload)?;
        match result {
            StatusResult::Status => {
                let packet = StatusResponsePacket::from_status_response(&StatusResponse::new(
                    "1.21.4",
                    769,
                    "A Minecraft Server",
                    false,
                ));
                self.write_packet(packet).await?;
            }
            StatusResult::Ping(timestamp) => {
                let packet = PingResponsePacket { timestamp };
                self.write_packet(packet).await?;
            }
        };
        Ok(())
    }

    async fn handle_handshake(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let (packet_id, packet_payload) = self.get_packet();
        let next_state = handle_handshake_state(packet_id, packet_payload)?;
        self.update_state(next_state);
        Ok(())
    }

    pub async fn send_keep_alive(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.state == State::Play {
            let packet = ClientBoundKeepAlivePacket::new(get_random());
            self.write_packet(packet).await
        } else {
            Ok(())
        }
    }

    async fn write_packet(
        &mut self,
        packet: impl EncodePacket + PacketId,
    ) -> Result<(), Box<dyn std::error::Error>> {
        debug!("writing packet id 0x{:02x}", packet.get_packet_id());
        let encoded_packet = packet.encode()?;
        let mut payload = Vec::new();
        VarInt::new(encoded_packet.len() as i32 + 1).encode(&mut payload)?;
        payload.push(packet.get_packet_id());
        payload.extend_from_slice(&encoded_packet);
        self.socket.write_all(&payload).await?;
        Ok(())
    }
}

#[allow(dead_code)]
pub fn print_bytes_hex(bytes: &[u8], length: usize) -> String {
    bytes[..length]
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<Vec<_>>()
        .join(" ")
}

fn get_random() -> i64 {
    let mut rng = rand::thread_rng();
    rng.gen()
}
