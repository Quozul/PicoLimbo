use crate::server::client_state::ClientState;
use crate::server::game_mode::GameMode;
use crate::server::packet_handler::{PacketHandler, PacketHandlerError};
use crate::server::packet_registry::PacketRegistry;
use crate::server_state::{ServerState, TabList};
use blocks_report::get_block_report_id_mapping;
use minecraft_packets::configuration::acknowledge_finish_configuration_packet::AcknowledgeConfigurationPacket;
use minecraft_packets::play::chunk_data_and_update_light_packet::ChunkDataAndUpdateLightPacket;
use minecraft_packets::play::commands_packet::CommandsPacket;
use minecraft_packets::play::game_event_packet::GameEventPacket;
use minecraft_packets::play::login_packet::LoginPacket;
use minecraft_packets::play::play_client_bound_plugin_message_packet::PlayClientBoundPluginMessagePacket;
use minecraft_packets::play::set_chunk_cache_center_packet::SetCenterChunkPacket;
use minecraft_packets::play::set_default_spawn_position_packet::SetDefaultSpawnPositionPacket;
use minecraft_packets::play::synchronize_player_position_packet::SynchronizePlayerPositionPacket;
use minecraft_packets::play::tab_list_packet::TabListPacket;
use minecraft_packets::play::update_time_packet::UpdateTimePacket;
use minecraft_packets::play::{VoidChunkContext, WorldContext};
use minecraft_protocol::prelude::{Coordinates, Dimension, ProtocolVersion, State};
use pico_structures::prelude::{SchematicError, World};
use pico_text_component::prelude::Component;
use registries::{Registries, get_dimension_index, get_registries, get_void_biome_index};
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

const F64_CONVERSION_FAILED: &str = "Conversion failed: Invalid or out-of-range float";

fn safe_f64_to_i32(f: f64) -> Option<i32> {
    if f.is_finite() && f >= f64::from(i32::MIN) && f <= f64::from(i32::MAX) {
        #[allow(clippy::cast_possible_truncation)]
        Some(f as i32)
    } else {
        None
    }
}

fn world_position_to_chunk_position(
    position: (f64, f64),
) -> Result<(i32, i32), PacketHandlerError> {
    let chunk_x = safe_f64_to_i32((position.0 / 16.0).floor())
        .ok_or_else(|| PacketHandlerError::invalid_state(F64_CONVERSION_FAILED))?;
    let chunk_z = safe_f64_to_i32((position.1 / 16.0).floor())
        .ok_or_else(|| PacketHandlerError::invalid_state(F64_CONVERSION_FAILED))?;
    Ok((chunk_x, chunk_z))
}

fn create_circular_chunk_iterator(
    center_chunk: (i32, i32),
    view_distance: i32,
) -> impl Iterator<Item = (i32, i32)> {
    let (center_x, center_z) = center_chunk;
    let mut offsets = Vec::new();
    for dx in -view_distance..=view_distance {
        for dz in -view_distance..=view_distance {
            offsets.push((dx, dz));
        }
    }

    // Sort by squared distance for efficiency (avoids sqrt)
    offsets.sort_unstable_by_key(|(dx, dz)| dx.pow(2) + dz.pow(2));

    offsets
        .into_iter()
        .map(move |(dx, dz)| (center_x + dx, center_z + dz))
}

fn send_chunks_circularly(
    client_state: &mut ClientState,
    center_chunk: (i32, i32),
    view_distance: i32,
    world: Option<&World>,
    biome_index: i32,
    dimension: Dimension,
    protocol_version: ProtocolVersion,
) {
    let chunk_positions = create_circular_chunk_iterator(center_chunk, view_distance);

    let paste_origin = Coordinates::new_uniform(0);
    let report_id_mapping = get_block_report_id_mapping(protocol_version);
    let schematic_context = world.and_then(|world| {
        report_id_mapping
            .as_ref()
            .map(|report_id_mapping| WorldContext {
                paste_origin,
                world,
                report_id_mapping,
            })
            .ok()
    });

    for (chunk_x, chunk_z) in chunk_positions {
        let chunk_context = VoidChunkContext {
            chunk_x,
            chunk_z,
            biome_index,
            dimension,
        };

        let packet = if let Some(ref context) = schematic_context {
            ChunkDataAndUpdateLightPacket::from_structure(chunk_context, context)
        } else {
            ChunkDataAndUpdateLightPacket::void(chunk_context)
        };

        client_state.queue_packet(PacketRegistry::ChunkDataAndUpdateLight(Box::new(packet)));
    }
}

impl From<SchematicError> for PacketHandlerError {
    fn from(value: SchematicError) -> Self {
        Self::Custom(value.to_string())
    }
}

pub fn send_play_packets(
    client_state: &mut ClientState,
    server_state: &ServerState,
) -> Result<(), PacketHandlerError> {
    let protocol_version = client_state.protocol_version();
    let view_distance = server_state.view_distance();
    let dimension = server_state.spawn_dimension();

    let game_mode = {
        let expected_game_mode = server_state.game_mode();
        let is_spectator = expected_game_mode == GameMode::Spectator;

        if protocol_version.is_before_inclusive(ProtocolVersion::V1_7_6) && is_spectator {
            GameMode::Creative
        } else {
            expected_game_mode
        }
    };

    let packet = build_login_packet(protocol_version, dimension)?
        .set_game_mode(game_mode.value())
        .set_view_distance(view_distance)
        .set_hardcore(protocol_version, server_state.is_hardcore());
    client_state.queue_packet(PacketRegistry::Login(Box::new(packet)));

    let (x, y, z) = server_state.spawn_position();
    if protocol_version.is_after_inclusive(ProtocolVersion::V1_19) {
        // Send Set Default Spawn Position
        let packet = SetDefaultSpawnPositionPacket::new(x, y, z);
        client_state.queue_packet(PacketRegistry::SetDefaultSpawnPosition(packet));
    }

    // Send Synchronize Player Position
    let packet = SynchronizePlayerPositionPacket::new(x, y, z);
    client_state.queue_packet(PacketRegistry::SynchronizePlayerPosition(packet));

    if protocol_version.is_after_inclusive(ProtocolVersion::V1_13) {
        let packet = CommandsPacket::empty();
        client_state.queue_packet(PacketRegistry::Commands(packet));
    }

    // The brand is not visible for clients prior to 1.13, no need to send it
    // The brand is sent during the configuration state after 1.20.2 included
    if protocol_version.between_inclusive(ProtocolVersion::V1_13, ProtocolVersion::V1_20) {
        let packet = PlayClientBoundPluginMessagePacket::brand("PicoLimbo");
        client_state.queue_packet(PacketRegistry::PlayClientBoundPluginMessage(packet));
    }

    if let Some(component) = server_state.welcome_message() {
        client_state.send_message(component);
    }

    let ticks = server_state.time_world_ticks();
    let lock_time = server_state.is_time_locked();
    let packet = UpdateTimePacket::new(ticks, ticks, !lock_time);
    client_state.queue_packet(PacketRegistry::UpdateTime(packet));

    match server_state.tab_list() {
        TabList::HeaderAndFooter { header, footer } => {
            let packet = TabListPacket::new(header, footer);
            client_state.queue_packet(PacketRegistry::TabList(packet));
        }
        TabList::Header { header } => {
            let empty = Component::default();
            let packet = TabListPacket::new(header, &empty);
            client_state.queue_packet(PacketRegistry::TabList(packet));
        }
        TabList::Footer { footer } => {
            let empty = Component::default();
            let packet = TabListPacket::new(&empty, footer);
            client_state.queue_packet(PacketRegistry::TabList(packet));
        }
        TabList::None => {}
    }

    if protocol_version.is_after_inclusive(ProtocolVersion::V1_19) {
        if protocol_version.is_after_inclusive(ProtocolVersion::V1_20_3) {
            // Send Game Event
            let packet = GameEventPacket::start_waiting_for_chunks(0.0);
            client_state.queue_packet(PacketRegistry::GameEvent(packet));
        }

        // Send Chunk Data and Update Light
        let biome_id = get_void_biome_index(protocol_version).ok_or_else(|| {
            PacketHandlerError::InvalidState(format!(
                "Cannot find void biome index for version {protocol_version}"
            ))
        })?;

        let center_chunk = world_position_to_chunk_position((x, z))?;
        let packet = SetCenterChunkPacket::new(center_chunk.0, center_chunk.1);
        client_state.queue_packet(PacketRegistry::SetCenterChunk(packet));

        send_chunks_circularly(
            client_state,
            center_chunk,
            view_distance,
            server_state.world(),
            biome_id,
            dimension,
            protocol_version,
        );
    }

    client_state.set_state(State::Play);
    client_state.set_keep_alive_should_enable();

    Ok(())
}

impl From<TryFromIntError> for PacketHandlerError {
    fn from(_: TryFromIntError) -> Self {
        Self::custom("failed to cast int")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn server_state() -> ServerState {
        let mut builder = ServerState::builder();
        builder.view_distance(0);
        builder.welcome_message("Hello, World!");
        builder.build().unwrap()
    }

    fn client(protocol: ProtocolVersion) -> ClientState {
        let mut cs = ClientState::default();
        cs.set_protocol_version(protocol);
        let previous_state = if protocol.supports_configuration_state() {
            State::Configuration
        } else {
            State::Login
        };
        cs.set_state(previous_state);
        cs
    }

    #[test]
    fn test_v1_20_3_play_packets() {
        // Given
        let mut client_state = client(ProtocolVersion::V1_20_3);
        let server_state = server_state();

        // When
        send_play_packets(&mut client_state, &server_state).unwrap();

        // Then
        assert!(matches!(
            client_state.next_packet(),
            PacketRegistry::Login(_)
        ));
        assert!(matches!(
            client_state.next_packet(),
            PacketRegistry::SetDefaultSpawnPosition(_)
        ));
        assert!(matches!(
            client_state.next_packet(),
            PacketRegistry::SynchronizePlayerPosition(_)
        ));
        assert!(matches!(
            client_state.next_packet(),
            PacketRegistry::Commands(_)
        ));
        assert!(matches!(
            client_state.next_packet(),
            PacketRegistry::SystemChatMessage(_)
        ));
        assert!(matches!(
            client_state.next_packet(),
            PacketRegistry::UpdateTime(_)
        ));
        assert!(matches!(
            client_state.next_packet(),
            PacketRegistry::GameEvent(_)
        ));
        assert!(matches!(
            client_state.next_packet(),
            PacketRegistry::SetCenterChunk(_)
        ));
        assert!(matches!(
            client_state.next_packet(),
            PacketRegistry::ChunkDataAndUpdateLight(_)
        ));
        assert!(client_state.has_no_more_packets());
    }

    #[test]
    fn test_v1_19_play_packets() {
        // Given
        let mut client_state = client(ProtocolVersion::V1_19);
        let server_state = server_state();

        // When
        send_play_packets(&mut client_state, &server_state).unwrap();

        // Then
        assert!(matches!(
            client_state.next_packet(),
            PacketRegistry::Login(_)
        ));
        assert!(matches!(
            client_state.next_packet(),
            PacketRegistry::SetDefaultSpawnPosition(_)
        ));
        assert!(matches!(
            client_state.next_packet(),
            PacketRegistry::SynchronizePlayerPosition(_)
        ));
        assert!(matches!(
            client_state.next_packet(),
            PacketRegistry::Commands(_)
        ));
        assert!(matches!(
            client_state.next_packet(),
            PacketRegistry::PlayClientBoundPluginMessage(_)
        ));
        assert!(matches!(
            client_state.next_packet(),
            PacketRegistry::SystemChatMessage(_)
        ));
        assert!(matches!(
            client_state.next_packet(),
            PacketRegistry::UpdateTime(_)
        ));
        assert!(matches!(
            client_state.next_packet(),
            PacketRegistry::SetCenterChunk(_)
        ));
        assert!(matches!(
            client_state.next_packet(),
            PacketRegistry::ChunkDataAndUpdateLight(_)
        ));
        assert!(client_state.has_no_more_packets());
    }

    #[test]
    fn test_v1_13_play_packets() {
        // Given
        let mut client_state = client(ProtocolVersion::V1_13);
        let server_state = server_state();

        // When
        send_play_packets(&mut client_state, &server_state).unwrap();

        // Then
        assert!(matches!(
            client_state.next_packet(),
            PacketRegistry::Login(_)
        ));
        assert!(matches!(
            client_state.next_packet(),
            PacketRegistry::SynchronizePlayerPosition(_)
        ));
        assert!(matches!(
            client_state.next_packet(),
            PacketRegistry::Commands(_)
        ));
        assert!(matches!(
            client_state.next_packet(),
            PacketRegistry::PlayClientBoundPluginMessage(_)
        ));
        assert!(matches!(
            client_state.next_packet(),
            PacketRegistry::LegacyChatMessage(_)
        ));
        assert!(matches!(
            client_state.next_packet(),
            PacketRegistry::UpdateTime(_)
        ));
        assert!(client_state.has_no_more_packets());
    }

    #[test]
    fn test_pre_modern_play_packets() {
        // Given
        let mut client_state = client(ProtocolVersion::V1_12_2);
        let server_state = server_state();

        // When
        send_play_packets(&mut client_state, &server_state).unwrap();

        // Then
        assert!(matches!(
            client_state.next_packet(),
            PacketRegistry::Login(_)
        ));
        assert!(matches!(
            client_state.next_packet(),
            PacketRegistry::SynchronizePlayerPosition(_)
        ));
        assert!(matches!(
            client_state.next_packet(),
            PacketRegistry::LegacyChatMessage(_)
        ));
        assert!(matches!(
            client_state.next_packet(),
            PacketRegistry::UpdateTime(_)
        ));
        assert!(client_state.has_no_more_packets());
    }
}
