use crate::server::client_state::ClientState;
use crate::server::game_mode::GameMode;
use crate::server::packet_handler::{PacketHandler, PacketHandlerError};
use crate::server::packet_registry::PacketRegistry;
use crate::server_state::ServerState;
use minecraft_packets::configuration::acknowledge_finish_configuration_packet::AcknowledgeConfigurationPacket;
use minecraft_packets::configuration::client_bound_known_packs_packet::ClientBoundKnownPacksPacket;
use minecraft_packets::configuration::configuration_client_bound_plugin_message_packet::ConfigurationClientBoundPluginMessagePacket;
use minecraft_packets::configuration::finish_configuration_packet::FinishConfigurationPacket;
use minecraft_packets::configuration::registry_data_packet::{
    RegistryDataCodecPacket, RegistryDataPacket,
};
use minecraft_packets::play::chunk_data_and_update_light_packet::ChunkDataAndUpdateLightPacket;
use minecraft_packets::play::commands_packet::CommandsPacket;
use minecraft_packets::play::game_event_packet::GameEventPacket;
use minecraft_packets::play::legacy_chat_message_packet::LegacyChatMessagePacket;
use minecraft_packets::play::login_packet::LoginPacket;
use minecraft_packets::play::play_client_bound_plugin_message_packet::PlayClientBoundPluginMessagePacket;
use minecraft_packets::play::set_default_spawn_position_packet::SetDefaultSpawnPositionPacket;
use minecraft_packets::play::synchronize_player_position_packet::SynchronizePlayerPositionPacket;
use minecraft_packets::play::system_chat_message_packet::SystemChatMessagePacket;
use minecraft_protocol::data::registry::get_all_registries::{
    get_dimension_type_index, get_the_void_index, get_v1_16_2_registry_codec,
    get_v1_16_registry_codec, get_v1_20_5_registries,
};
use minecraft_protocol::prelude::{LengthPaddedVec, Nbt, ProtocolVersion};
use minecraft_protocol::state::State;
use std::num::TryFromIntError;
use thiserror::Error;

impl PacketHandler for AcknowledgeConfigurationPacket {
    fn handle(
        &self,
        client_state: &mut ClientState,
        server_state: &ServerState,
    ) -> Result<(), PacketHandlerError> {
        send_play_packets(client_state, server_state)
    }
}

/// Only for <= 1.20.2
pub fn send_configuration_packets(
    client_state: &mut ClientState,
    server_state: &ServerState,
) -> Result<(), PacketHandlerError> {
    // Send Server Brand
    let packet = ConfigurationClientBoundPluginMessagePacket::brand("PicoLimbo");
    client_state.queue_packet(PacketRegistry::ConfigurationClientBoundPluginMessage(
        packet,
    ))?;
    let protocol_version = client_state.protocol_version();

    if ProtocolVersion::V1_20_5 <= protocol_version {
        // Send Known Packs
        let packet = ClientBoundKnownPacksPacket::default();
        client_state.queue_packet(PacketRegistry::ClientBoundKnownPacks(packet))?;

        // Send Registry Data
        let grouped = get_v1_20_5_registries(protocol_version, server_state.data_directory());
        for (registry_id, entries) in grouped {
            let packet = RegistryDataPacket {
                registry_id,
                entries: LengthPaddedVec::new(entries),
            };
            client_state.queue_packet(PacketRegistry::RegistryData(packet))?;
        }
    } else {
        // Only for 1.20.2 and 1.20.3
        let registry_codec =
            get_v1_16_2_registry_codec(protocol_version, server_state.data_directory());
        let packet = RegistryDataCodecPacket { registry_codec };
        client_state.queue_packet(PacketRegistry::RegistryDataCodec(packet))?;
    }

    // Send Finished Configuration
    let packet = FinishConfigurationPacket {};
    client_state.queue_packet(PacketRegistry::FinishConfiguration(packet))?;

    Ok(())
}

/// Switch to the Play state and send required packets to spawn the player in the world
pub fn send_play_packets(
    client_state: &mut ClientState,
    server_state: &ServerState,
) -> Result<(), PacketHandlerError> {
    let protocol_version = client_state.protocol_version();

    let dimension_type = get_dimension_type_index(
        protocol_version,
        server_state.data_directory(),
        server_state.spawn_dimension().identifier().thing,
    );
    let dimension_type = i32::try_from(dimension_type)?;

    let packet =
        if protocol_version.between_inclusive(ProtocolVersion::V1_16, ProtocolVersion::V1_20) {
            match construct_registry_data(protocol_version, server_state) {
                Ok((registry_codec, dimension)) => LoginPacket::new_with_codecs(
                    server_state.spawn_dimension(),
                    registry_codec,
                    dimension,
                    dimension_type,
                )
                .set_game_mode(server_state.game_mode().value()),
                Err(e) => {
                    client_state.kick("Disconnected");
                    return Err(PacketHandlerError::custom(&e.to_string()));
                }
            }
        } else {
            let game_mode = {
                let expected_game_mode = server_state.game_mode();
                let is_spectator = expected_game_mode == GameMode::Spectator;

                if protocol_version.before_inclusive(ProtocolVersion::V1_7_6) && is_spectator {
                    GameMode::Creative
                } else {
                    expected_game_mode
                }
            };

            LoginPacket::new_with_dimension(server_state.spawn_dimension(), dimension_type)
                .set_game_mode(game_mode.value())
        };
    client_state.queue_packet(PacketRegistry::Login(Box::new(packet)))?;

    // Send Synchronize Player Position
    let packet = SynchronizePlayerPositionPacket::default();
    client_state.queue_packet(PacketRegistry::SynchronizePlayerPosition(packet))?;

    if protocol_version >= ProtocolVersion::V1_19 {
        // Send Set Default Spawn Position
        let packet = SetDefaultSpawnPositionPacket::default();
        client_state.queue_packet(PacketRegistry::SetDefaultSpawnPosition(packet))?;
    }

    if protocol_version >= ProtocolVersion::V1_13 {
        let packet = CommandsPacket::empty();
        client_state.queue_packet(PacketRegistry::Commands(packet))?;
    }

    if protocol_version >= ProtocolVersion::V1_20_3 {
        // Send Game Event
        let packet = GameEventPacket::start_waiting_for_chunks(0.0);
        client_state.queue_packet(PacketRegistry::GameEvent(packet))?;

        // Send Chunk Data and Update Light
        let biome_id = get_the_void_index(protocol_version, server_state.data_directory());
        let biome_id = i32::try_from(biome_id)?;
        let packet = ChunkDataAndUpdateLightPacket::new(protocol_version, biome_id);
        client_state.queue_packet(PacketRegistry::ChunkDataAndUpdateLight(packet))?;
    }

    client_state.set_state(State::Play);
    client_state.set_keep_alive_should_enable();

    // The brand is not visible for clients prior to 1.13, no need to send it
    // The brand is sent during the configuration state after 1.20.2 included
    if protocol_version.between_inclusive(ProtocolVersion::V1_13, ProtocolVersion::V1_20) {
        let packet = PlayClientBoundPluginMessagePacket::brand("PicoLimbo");
        client_state.queue_packet(PacketRegistry::PlayClientBoundPluginMessage(packet))?;
    }

    if let Some(content) = server_state.welcome_message() {
        if protocol_version >= ProtocolVersion::V1_19 {
            let packet = SystemChatMessagePacket::plain_text(content);
            client_state.queue_packet(PacketRegistry::SystemChatMessage(packet))?;
        } else {
            let packet = LegacyChatMessagePacket::system(content);
            client_state.queue_packet(PacketRegistry::LegacyChatMessage(packet))?;
        }
    }

    Ok(())
}

fn construct_registry_data(
    protocol_version: ProtocolVersion,
    state: &ServerState,
) -> Result<(Nbt, Nbt), RegistryError> {
    let registry_codec = if protocol_version == ProtocolVersion::V1_16
        || protocol_version == ProtocolVersion::V1_16_1
    {
        get_v1_16_registry_codec(state.data_directory())
            .map_err(|_| RegistryError::CodecConstruction)?
    } else {
        get_v1_16_2_registry_codec(protocol_version, state.data_directory())
    };

    // For versions between 1.16.2 and 1.18.2 (included), we must send the dimension codec separately
    let dimension =
        if protocol_version.between_inclusive(ProtocolVersion::V1_16_2, ProtocolVersion::V1_18_2) {
            let dimension_types = registry_codec
                .find_tag("minecraft:dimension_type")
                .ok_or(RegistryError::MissingDimensionType)?
                .find_tag("value")
                .ok_or(RegistryError::MissingValue)?
                .get_vec()
                .ok_or(RegistryError::InvalidVecFormat)?;

            let dimension = dimension_types
                .iter()
                .find(|element| {
                    element
                        .find_tag("name".to_string())
                        .is_some_and(|name| match name {
                            Nbt::String { value, .. } => {
                                value == &state.spawn_dimension().identifier().to_string()
                            }
                            _ => false,
                        })
                })
                .cloned()
                .unwrap_or_else(|| dimension_types.first().cloned().unwrap_or(Nbt::End));

            dimension
                .find_tag("element")
                .ok_or(RegistryError::MissingElement)?
                .clone()
        } else {
            Nbt::End
        };

    Ok((registry_codec, dimension))
}

#[derive(Debug, Error)]
pub enum RegistryError {
    #[error("Failed to construct registry codec")]
    CodecConstruction,
    #[error("Missing dimension type in registry")]
    MissingDimensionType,
    #[error("Missing value tag in registry")]
    MissingValue,
    #[error("Invalid vector format in registry")]
    InvalidVecFormat,
    #[error("Missing element tag in dimension")]
    MissingElement,
}

impl From<TryFromIntError> for PacketHandlerError {
    fn from(_: TryFromIntError) -> Self {
        Self::custom("failed to cast int")
    }
}
