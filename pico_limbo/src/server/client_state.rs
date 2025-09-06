use crate::server::fifo::Fifo;
use crate::server::game_profile::GameProfile;
use crate::server::packet_registry::PacketRegistry;
use minecraft_packets::play::legacy_chat_message_packet::LegacyChatMessagePacket;
use minecraft_packets::play::system_chat_message_packet::SystemChatMessagePacket;
use minecraft_protocol::prelude::{ProtocolVersion, State};
use tracing::info;
use minecraft_packets::play::legacy_chat_message_packet::LegacyChatMessagePacket;
use minecraft_packets::play::system_chat_message_packet::SystemChatMessagePacket;

#[derive(PartialEq, Eq)]
pub enum KeepAliveStatus {
    Disabled,
    ShouldEnable,
    Enabled,
}

impl Default for ClientState {
    fn default() -> Self {
        Self {
            state: State::Handshake,
            protocol_version: ProtocolVersion::Any,
            kick_message: None,
            pending_packets: Fifo::new(),
            message_id: -1,
            game_profile: None,
            keep_alive_enabled: KeepAliveStatus::Disabled,
        }
    }
}

pub struct ClientState {
    state: State,
    protocol_version: ProtocolVersion,
    kick_message: Option<String>,
    pending_packets: Fifo<PacketRegistry>,
    message_id: i32,
    game_profile: Option<GameProfile>,
    keep_alive_enabled: KeepAliveStatus,
}

impl ClientState {
    const ANONYMOUS: &'static str = "Anonymous";

    // Kick

    pub fn kick(&mut self, kick_message: &str) {
        self.kick_message = Some(kick_message.to_string());
    }

    pub fn should_kick(&self) -> Option<String> {
        self.kick_message.clone()
    }

    // State

    pub const fn state(&self) -> State {
        self.state
    }

    pub const fn set_state(&mut self, new_state: State) {
        self.state = new_state;
    }

    // Protocol version

    pub const fn protocol_version(&self) -> ProtocolVersion {
        self.protocol_version
    }

    pub const fn set_protocol_version(&mut self, new_protocol_version: ProtocolVersion) {
        self.protocol_version = new_protocol_version;
    }

    // Packets

    pub fn queue_packet(&mut self, packet: PacketRegistry) {
        self.pending_packets.push(packet);
    }

    pub const fn pending_packets(&mut self) -> &mut Fifo<PacketRegistry> {
        &mut self.pending_packets
    }

    #[cfg(test)]
    pub fn next_packet(&mut self) -> PacketRegistry {
        self.pending_packets.pop().unwrap()
    }

    #[cfg(test)]
    pub fn has_no_more_packets(&self) -> bool {
        self.pending_packets.is_empty()
    }

    // Velocity

    pub const fn set_velocity_login_message_id(&mut self, message_id: i32) {
        self.message_id = message_id;
    }

    pub const fn get_velocity_login_message_id(&self) -> i32 {
        self.message_id
    }

    // Game profile

    pub fn set_game_profile(&mut self, game_profile: GameProfile) {
        info!(
            "UUID of player {} is {}",
            game_profile.username(),
            game_profile.uuid()
        );
        self.game_profile = Some(game_profile);
    }

    pub fn game_profile(&self) -> Option<GameProfile> {
        self.game_profile.clone()
    }

    pub fn get_username(&self) -> String {
        self.game_profile()
            .map_or(Self::ANONYMOUS.to_owned(), |profile| {
                profile.username().to_owned()
            })
    }

    // Keep alive

    pub fn should_enable_keep_alive(&self) -> bool {
        self.keep_alive_enabled == KeepAliveStatus::ShouldEnable
    }

    pub fn set_keep_alive_should_enable(&mut self) {
        if self.keep_alive_enabled == KeepAliveStatus::Disabled {
            self.keep_alive_enabled = KeepAliveStatus::ShouldEnable;
        }
    }

    pub fn set_keep_alive_enabled(&mut self) {
        if self.keep_alive_enabled == KeepAliveStatus::ShouldEnable {
            self.keep_alive_enabled = KeepAliveStatus::Enabled;
        }
    }

    pub fn send_message(&mut self, message: String) {
        if self.protocol_version.is_after_inclusive(ProtocolVersion::V1_19) {
            let packet = SystemChatMessagePacket::plain_text(message);
            self.queue_packet(PacketRegistry::SystemChatMessage(packet));
        } else {
            let packet = LegacyChatMessagePacket::system(message);
            self.queue_packet(PacketRegistry::LegacyChatMessage(packet));
        }
    }
}
