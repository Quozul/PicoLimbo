use crate::server::client_state::ClientState;
use crate::server::game_mode::GameMode;
use crate::server::packet_handler::{PacketHandler, PacketHandlerError};
use crate::server::packet_registry::PacketRegistry;
use crate::server_state::ServerState;
use minecraft_packets::configuration::acknowledge_finish_configuration_packet::AcknowledgeConfigurationPacket;
use minecraft_packets::configuration::client_bound_known_packs_packet::ClientBoundKnownPacksPacket;
use minecraft_packets::configuration::configuration_client_bound_plugin_message_packet::ConfigurationClientBoundPluginMessagePacket;
use minecraft_packets::configuration::finish_configuration_packet::FinishConfigurationPacket;
use minecraft_packets::configuration::registry_data_packet::RegistryDataPacket;
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
    Registries, get_dimension_type_index, get_registries, get_the_void_index,
};
use minecraft_protocol::data::registry::registry_entry::RegistryEntry;
use minecraft_protocol::prelude::{Omitted, ProtocolVersion};
use minecraft_protocol::state::State;
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

    if protocol_version.is_after_inclusive(ProtocolVersion::V1_20_5) {
        // Send Known Packs
        let packet = ClientBoundKnownPacksPacket::default();
        client_state.queue_packet(PacketRegistry::ClientBoundKnownPacks(packet))?;
    }

    // Send Registry Data
    match get_registries(
        protocol_version,
        server_state.data_directory(),
        server_state.spawn_dimension().identifier().to_string(),
    ) {
        Registries::V1_20_5 { registries } => {
            for (registry_id, entries) in registries {
                let registry_entries = entries
                    .iter()
                    .map(|(entry_id, bytes)| RegistryEntry::new(entry_id.clone(), bytes.clone()))
                    .collect();
                let packet = RegistryDataPacket::registry(registry_id, registry_entries);
                client_state.queue_packet(PacketRegistry::RegistryData(packet))?;
            }
        }
        Registries::V1_20_2 { registry_codec } => {
            let packet = RegistryDataPacket::codec(registry_codec);
            client_state.queue_packet(PacketRegistry::RegistryData(packet))?;
        }
        _ => {
            client_state.kick("Disconnected");
            return Err(PacketHandlerError::custom(
                "Registry not supported for this version",
            ));
        }
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

    let game_mode = {
        let expected_game_mode = server_state.game_mode();
        let is_spectator = expected_game_mode == GameMode::Spectator;

        if protocol_version.before_inclusive(ProtocolVersion::V1_7_6) && is_spectator {
            GameMode::Creative
        } else {
            expected_game_mode
        }
    };

    let packet = match get_registries(
        protocol_version,
        server_state.data_directory(),
        server_state.spawn_dimension().identifier().to_string(),
    ) {
        Registries::V1_16_2 {
            registry_codec,
            dimension,
        } => LoginPacket::new_with_codecs(
            server_state.spawn_dimension(),
            Omitted::Some(registry_codec),
            Omitted::Some(dimension),
            dimension_type,
        ),
        Registries::V1_19 { registry_codec } | Registries::V1_16 { registry_codec } => {
            LoginPacket::new_with_codecs(
                server_state.spawn_dimension(),
                Omitted::Some(registry_codec),
                Omitted::None,
                dimension_type,
            )
        }
        _ => LoginPacket::new_with_dimension(server_state.spawn_dimension(), dimension_type),
    }
    .set_game_mode(game_mode.value());

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

impl From<TryFromIntError> for PacketHandlerError {
    fn from(_: TryFromIntError) -> Self {
        Self::custom("failed to cast int")
    }
}
