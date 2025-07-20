use crate::server::client_inner::{ClientInner, ClientReadPacketError, ClientSendPacketError};
use crate::server::controllable_interval::ControllableInterval;
use crate::server::game_profile::GameProfile;
use crate::server::named_packet::NamedPacket;
use minecraft_packets::login::login_disconnect_packet::LoginDisconnectPacket;
use minecraft_packets::play::client_bound_keep_alive_packet::ClientBoundKeepAlivePacket;
use minecraft_protocol::data::packets_report::packet_map::PacketMap;
use minecraft_protocol::prelude::{EncodePacket, PacketId};
use minecraft_protocol::protocol_version::ProtocolVersion;
use minecraft_protocol::state::State;
use rand::Rng;
use std::ops::Add;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio::time::Instant;

#[derive(Clone)]
pub struct Client {
    inner: Arc<Mutex<ClientInner>>,
    interval: Arc<Mutex<ControllableInterval>>,
}

impl Client {
    const ANONYMOUS: &'static str = "Anonymous";

    pub fn new(socket: TcpStream, packet_map: PacketMap) -> Self {
        Self {
            inner: Arc::new(Mutex::new(ClientInner::new(socket, packet_map))),
            interval: Arc::new(Mutex::new(ControllableInterval::new())),
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
        drop(inner);
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
        drop(inner);
        Ok(())
    }

    pub async fn shutdown(&self) -> std::io::Result<()> {
        self.acquire_lock().await.shutdown().await?;
        self.interval().await.clear_interval().await;
        Ok(())
    }

    pub async fn set_game_profile(&self, profile: GameProfile) {
        self.acquire_lock().await.set_game_profile_inner(profile);
    }

    pub async fn game_profile(&self) -> Option<GameProfile> {
        self.acquire_lock().await.game_profile_inner().cloned()
    }

    pub async fn get_username(&self) -> String {
        self.game_profile().await.map_or_else(
            || Self::ANONYMOUS.to_owned(),
            |profile| profile.username().to_owned(),
        )
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

    pub async fn is_any_version(&self) -> bool {
        self.protocol_version().await == ProtocolVersion::Any
    }

    pub async fn enable_keep_alive(&self) {
        if self
            .protocol_version()
            .await
            .before_inclusive(ProtocolVersion::V1_7_6)
        {
            let start = Instant::now().add(Duration::from_secs(2));
            let period = Duration::from_secs(2);
            self.interval().await.set_interval_at(start, period).await;
        } else {
            let period = Duration::from_secs(20);
            self.interval().await.set_interval(period).await;
        }
    }

    pub async fn keep_alive_tick(&self) {
        self.interval().await.tick().await;
    }

    #[inline]
    async fn acquire_lock(&self) -> tokio::sync::MutexGuard<'_, ClientInner> {
        self.inner.lock().await
    }

    #[inline]
    async fn interval(&self) -> tokio::sync::MutexGuard<'_, ControllableInterval> {
        self.interval.lock().await
    }
}

fn get_random_i64() -> i64 {
    let mut rng = rand::rng();
    rng.random()
}
