use crate::registries::get_registries::{
    Registries, get_dimension_index, get_registries, get_void_biome_index,
};
use crate::server::client_state::ClientState;
use crate::server::game_mode::GameMode;
use crate::server::packet_handler::{PacketHandler, PacketHandlerError};
use crate::server::packet_registry::PacketRegistry;
use crate::server_state::ServerState;
use minecraft_packets::configuration::acknowledge_finish_configuration_packet::AcknowledgeConfigurationPacket;
use minecraft_packets::play::chunk_data_and_update_light_packet::ChunkDataAndUpdateLightPacket;
use minecraft_packets::play::commands_packet::CommandsPacket;
use minecraft_packets::play::game_event_packet::GameEventPacket;
use minecraft_packets::play::legacy_chat_message_packet::LegacyChatMessagePacket;
use minecraft_packets::play::login_packet::LoginPacket;
use minecraft_packets::play::play_client_bound_plugin_message_packet::PlayClientBoundPluginMessagePacket;
use minecraft_packets::play::set_default_spawn_position_packet::SetDefaultSpawnPositionPacket;
use minecraft_packets::play::synchronize_player_position_packet::SynchronizePlayerPositionPacket;
use minecraft_packets::play::system_chat_message_packet::SystemChatMessagePacket;
use minecraft_protocol::prelude::{Dimension, ProtocolVersion, State};
use std::num::TryFromIntError;

impl PacketHandler for AcknowledgeConfigurationPacket {
    fn handle(
        &self,
        client_state: &mut ClientState,
        server_state: &ServerState,
    ) -> Result<(), PacketHandlerError> {
        send_play_packets(client_state, server_state)
    }
}

fn build_login_packet(
    protocol_version: ProtocolVersion,
    spawn_dimension: Dimension,
) -> Result<LoginPacket, PacketHandlerError> {
    if protocol_version.between_inclusive(ProtocolVersion::V1_7_2, ProtocolVersion::V1_15_2) {
        Ok(LoginPacket::with_dimension(spawn_dimension))
    } else if protocol_version.between_inclusive(ProtocolVersion::V1_16, ProtocolVersion::V1_20) {
        // We only need the registries here from 1.16 up to 1.20 included
        match get_registries(protocol_version, spawn_dimension) {
            Registries::V1_19 { registry_codec } | Registries::V1_16 { registry_codec } => Ok(
                LoginPacket::with_registry_codec(spawn_dimension, registry_codec),
            ),
            Registries::V1_16_2 {
                registry_codec,
                dimension,
            } => Ok(LoginPacket::with_dimension_codec(
                spawn_dimension,
                registry_codec,
                dimension,
            )),
            _ => unreachable!(),
        }
    } else if protocol_version.between_inclusive(ProtocolVersion::V1_20_2, ProtocolVersion::V1_20_3)
    {
        Ok(LoginPacket::with_dimension(spawn_dimension))
    } else if protocol_version.is_after_inclusive(ProtocolVersion::V1_20_5) {
        get_dimension_index(protocol_version, spawn_dimension).map_or_else(
            || {
                Err(PacketHandlerError::InvalidState(format!(
                    "Dimension index was not found for version {protocol_version}",
                )))
            },
            |dimension_index| {
                Ok(LoginPacket::with_dimension_index(
                    spawn_dimension,
                    dimension_index,
                ))
            },
        )
    } else {
        Err(PacketHandlerError::InvalidState(format!(
            "Cannot build login packet for version {protocol_version}",
        )))
    }
}

/// Switch to the Play state and send required packets to spawn the player in the world
pub fn send_play_packets(
    client_state: &mut ClientState,
    server_state: &ServerState,
) -> Result<(), PacketHandlerError> {
    let protocol_version = client_state.protocol_version();

    let game_mode = {
        let expected_game_mode = server_state.game_mode();
        let is_spectator = expected_game_mode == GameMode::Spectator;

        if protocol_version.is_before_inclusive(ProtocolVersion::V1_7_6) && is_spectator {
            GameMode::Creative
        } else {
            expected_game_mode
        }
    };

    let packet = build_login_packet(protocol_version, server_state.spawn_dimension())?
        .set_game_mode(game_mode.value());

    client_state.queue_packet(PacketRegistry::Login(Box::new(packet)));

    // Send Synchronize Player Position
    let packet = SynchronizePlayerPositionPacket::default();
    client_state.queue_packet(PacketRegistry::SynchronizePlayerPosition(packet));

    if protocol_version >= ProtocolVersion::V1_19 {
        // Send Set Default Spawn Position
        let packet = SetDefaultSpawnPositionPacket::default();
        client_state.queue_packet(PacketRegistry::SetDefaultSpawnPosition(packet));
    }

    if protocol_version >= ProtocolVersion::V1_13 {
        let packet = CommandsPacket::empty();
        client_state.queue_packet(PacketRegistry::Commands(packet));
    }

    if protocol_version >= ProtocolVersion::V1_20_3 {
        // Send Game Event
        let packet = GameEventPacket::start_waiting_for_chunks(0.0);
        client_state.queue_packet(PacketRegistry::GameEvent(packet));

        // Send Chunk Data and Update Light
        match get_void_biome_index(protocol_version) {
            Some(biome_id) => {
                let packet = ChunkDataAndUpdateLightPacket::new(protocol_version, biome_id);
                client_state.queue_packet(PacketRegistry::ChunkDataAndUpdateLight(packet));
            }
            None => {
                return Err(PacketHandlerError::InvalidState(format!(
                    "Cannot find void biome index for version {protocol_version}"
                )));
            }
        }
    }

    client_state.set_state(State::Play);
    client_state.set_keep_alive_should_enable();

    // The brand is not visible for clients prior to 1.13, no need to send it
    // The brand is sent during the configuration state after 1.20.2 included
    if protocol_version.between_inclusive(ProtocolVersion::V1_13, ProtocolVersion::V1_20) {
        let packet = PlayClientBoundPluginMessagePacket::brand("PicoLimbo");
        client_state.queue_packet(PacketRegistry::PlayClientBoundPluginMessage(packet));
    }

    if let Some(content) = server_state.welcome_message() {
        if protocol_version >= ProtocolVersion::V1_19 {
            let packet = SystemChatMessagePacket::plain_text(content);
            client_state.queue_packet(PacketRegistry::SystemChatMessage(packet));
        } else {
            let packet = LegacyChatMessagePacket::system(content);
            client_state.queue_packet(PacketRegistry::LegacyChatMessage(packet));
        }
    }

    Ok(())
}

impl From<TryFromIntError> for PacketHandlerError {
    fn from(_: TryFromIntError) -> Self {
        Self::custom("failed to cast int")
    }
}
