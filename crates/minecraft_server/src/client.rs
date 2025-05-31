use crate::client_inner::{ClientInner, ClientReadPacketError, ClientSendPacketError};
use crate::game_profile::GameProfile;
use crate::named_packet::NamedPacket;
use minecraft_packets::login::login_disconnect_packet::LoginDisconnectPacket;
use minecraft_packets::play::client_bound_keep_alive_packet::ClientBoundKeepAlivePacket;
use minecraft_protocol::data::packets_report::packet_map::PacketMap;
use minecraft_protocol::prelude::{EncodePacket, PacketId};
use minecraft_protocol::protocol_version::ProtocolVersion;
use minecraft_protocol::state::State;
use rand::Rng;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct Client {
    inner: Arc<Mutex<ClientInner>>,
}

impl Client {
    const ANONYMOUS: &'static str = "Anonymous";

    pub fn new(socket: TcpStream, packet_map: PacketMap) -> Self {
        Self {
            inner: Arc::new(Mutex::new(ClientInner::new(socket, packet_map))),
        }
    }

    pub async fn read_named_packet(&self) -> Result<NamedPacket, ClientReadPacketError> {
        let mut guard = self.acquire_lock().await;
        guard.read_named_packet_inner().await
    }

    pub async fn send_packet(
        &self,
        packet: impl EncodePacket + PacketId + Send,
    ) -> Result<(), ClientSendPacketError> {
        let mut guard = self.acquire_lock().await;
        guard.send_encodable_packet_inner(packet).await
    }

    pub async fn current_state(&self) -> State {
        self.acquire_lock().await.current_state().clone()
    }

    pub async fn set_state(&self, new_state: State) {
        self.acquire_lock().await.set_state(new_state);
    }

    pub async fn send_keep_alive(&self) -> Result<(), ClientSendPacketError> {
        let mut inner = self.acquire_lock().await;
        if inner.current_state() == &State::Play {
            let packet = ClientBoundKeepAlivePacket::new(get_random_i64());
            inner.send_encodable_packet_inner(packet).await?;
        }
        Ok(())
    }

    pub async fn kick<T>(&self, reason: T) -> Result<(), ClientSendPacketError>
    where
        T: Into<String>,
    {
        let mut inner = self.acquire_lock().await;
        if inner.current_state() == &State::Login {
            let packet = LoginDisconnectPacket::text(reason);
            inner.send_encodable_packet_inner(packet).await?;
        }
        inner.shutdown().await?;
        Ok(())
    }

    pub async fn set_game_profile(&self, profile: GameProfile) {
        self.acquire_lock().await.set_game_profile_inner(profile);
    }

    pub async fn game_profile(&self) -> Option<GameProfile> {
        self.acquire_lock().await.game_profile_inner().cloned()
    }

    pub async fn get_username(&self) -> String {
        self.game_profile()
            .await
            .map(|profile| profile.username().to_owned())
            .unwrap_or(Self::ANONYMOUS.to_owned())
    }

    pub async fn set_protocol_version(&self, protocol_version: ProtocolVersion) {
        self.acquire_lock()
            .await
            .set_protocol_inner(protocol_version);
    }

    pub async fn protocol_version(&self) -> ProtocolVersion {
        self.acquire_lock()
            .await
            .protocol_version_inner()
            .unwrap_or_default()
    }

    pub async fn set_velocity_login_message_id(&self, message_id: i32) {
        self.acquire_lock()
            .await
            .set_velocity_login_message_id_inner(message_id);
    }

    pub async fn get_velocity_login_message_id(&self) -> i32 {
        self.acquire_lock()
            .await
            .get_velocity_login_message_id_inner()
    }

    #[inline]
    async fn acquire_lock(&self) -> tokio::sync::MutexGuard<'_, ClientInner> {
        self.inner.lock().await
    }
}

fn get_random_i64() -> i64 {
    let mut rng = rand::rng();
    rng.random()
}
