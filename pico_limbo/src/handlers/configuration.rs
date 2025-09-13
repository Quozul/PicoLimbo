use crate::configuration::boss_bar::BossBarDivisionSerde;
use crate::handlers::play::fetch_minecraft_profile::fetch_minecraft_profile;
use crate::handlers::play::send_chunks_circularly::CircularChunkPacketIterator;
use crate::server::batch::Batch;
use crate::server::client_state::ClientState;
use crate::server::game_mode::GameMode;
use crate::server::packet_handler::{PacketHandler, PacketHandlerError};
use crate::server::packet_registry::PacketRegistry;
use crate::server_state::{BossBar, ServerState, TabList};
use minecraft_packets::configuration::acknowledge_finish_configuration_packet::AcknowledgeConfigurationPacket;
use minecraft_packets::login::Property;
use minecraft_packets::play::boss_bar_packet::{BossBarAction, BossBarPacket};
use minecraft_packets::play::commands_packet::CommandsPacket;
use minecraft_packets::play::game_event_packet::GameEventPacket;
use minecraft_packets::play::legacy_chat_message_packet::LegacyChatMessagePacket;
use minecraft_packets::play::login_packet::LoginPacket;
use minecraft_packets::play::play_client_bound_plugin_message_packet::PlayClientBoundPluginMessagePacket;
use minecraft_packets::play::player_info_update_packet::PlayerInfoUpdatePacket;
use minecraft_packets::play::set_chunk_cache_center_packet::SetCenterChunkPacket;
use minecraft_packets::play::set_default_spawn_position_packet::SetDefaultSpawnPositionPacket;
use minecraft_packets::play::set_entity_data_packet::SetEntityMetadataPacket;
use minecraft_packets::play::synchronize_player_position_packet::SynchronizePlayerPositionPacket;
use minecraft_packets::play::system_chat_message_packet::SystemChatMessagePacket;
use minecraft_packets::play::tab_list_packet::TabListPacket;
use minecraft_packets::play::update_time_packet::UpdateTimePacket;
use minecraft_protocol::prelude::{Dimension, ProtocolVersion, State, Uuid};
use pico_structures::prelude::SchematicError;
use pico_text_component::prelude::Component;
use registries::{Registries, get_dimension_index, get_registries, get_void_biome_index};
use std::num::TryFromIntError;
use tracing::debug;

impl PacketHandler for AcknowledgeConfigurationPacket {
    fn handle(
        &self,
        client_state: &mut ClientState,
        server_state: &ServerState,
    ) -> Result<Batch<PacketRegistry>, PacketHandlerError> {
        let mut batch = Batch::new();
        send_play_packets(&mut batch, client_state, server_state)?;
        Ok(batch)
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

impl From<SchematicError> for PacketHandlerError {
    fn from(value: SchematicError) -> Self {
        Self::Custom(value.to_string())
    }
}

pub fn send_play_packets(
    batch: &mut Batch<PacketRegistry>,
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
    batch.queue(|| PacketRegistry::Login(Box::new(packet)));

    let (x, y, z) = server_state.spawn_position();
    if protocol_version.is_after_inclusive(ProtocolVersion::V1_19) {
        // Send Set Default Spawn Position
        let packet = SetDefaultSpawnPositionPacket::new(x, y, z);
        batch.queue(|| PacketRegistry::SetDefaultSpawnPosition(packet));
    }

    // Send Synchronize Player Position
    let packet = SynchronizePlayerPositionPacket::new(x, y, z);
    batch.queue(|| PacketRegistry::SynchronizePlayerPosition(packet));

    if protocol_version.is_after_inclusive(ProtocolVersion::V1_13) {
        let packet = CommandsPacket::empty();
        batch.queue(|| PacketRegistry::Commands(packet));
    }

    // The brand is not visible for clients prior to 1.13, no need to send it
    // The brand is sent during the configuration state after 1.20.2 included
    if protocol_version.between_inclusive(ProtocolVersion::V1_13, ProtocolVersion::V1_20) {
        let packet = PlayClientBoundPluginMessagePacket::brand("PicoLimbo");
        batch.queue(|| PacketRegistry::PlayClientBoundPluginMessage(packet));
    }

    if let Some(component) = server_state.welcome_message() {
        send_message(batch, component, protocol_version);
    }

    let ticks = server_state.time_world_ticks();
    let lock_time = server_state.is_time_locked();
    let packet = UpdateTimePacket::new(ticks, ticks, !lock_time);
    batch.queue(|| PacketRegistry::UpdateTime(packet));

    send_tab_list_packets(batch, server_state);
    send_skin_packets(batch, client_state, server_state);
    send_boss_bar_packets(batch, server_state);

    if protocol_version.is_after_inclusive(ProtocolVersion::V1_19) {
        if protocol_version.is_after_inclusive(ProtocolVersion::V1_20_3) {
            // Send Game Event
            let packet = GameEventPacket::start_waiting_for_chunks(0.0);
            batch.queue(|| PacketRegistry::GameEvent(packet));
        }

        // Send Chunk Data and Update Light
        let biome_id = get_void_biome_index(protocol_version).ok_or_else(|| {
            PacketHandlerError::InvalidState(format!(
                "Cannot find void biome index for version {protocol_version}"
            ))
        })?;

        let center_chunk = world_position_to_chunk_position((x, z))?;
        let packet = SetCenterChunkPacket::new(center_chunk.0, center_chunk.1);
        batch.queue(|| PacketRegistry::SetCenterChunk(packet));

        let iter = CircularChunkPacketIterator::new(
            center_chunk,
            view_distance,
            server_state.world(),
            biome_id,
            dimension,
            protocol_version,
        );
        batch.chain_iter(iter);
    }

    client_state.set_state(State::Play);
    client_state.set_keep_alive_should_enable();

    Ok(())
}

fn send_tab_list_packets(batch: &mut Batch<PacketRegistry>, server_state: &ServerState) {
    match server_state.tab_list() {
        TabList::HeaderAndFooter { header, footer } => {
            let packet = TabListPacket::new(header, footer);
            batch.queue(|| PacketRegistry::TabList(packet));
        }
        TabList::Header { header } => {
            let empty = Component::default();
            let packet = TabListPacket::new(header, &empty);
            batch.queue(|| PacketRegistry::TabList(packet));
        }
        TabList::Footer { footer } => {
            let empty = Component::default();
            let packet = TabListPacket::new(&empty, footer);
            batch.queue(|| PacketRegistry::TabList(packet));
        }
        TabList::None => {}
    }
}

fn send_boss_bar_packets(batch: &mut Batch<PacketRegistry>, server_state: &ServerState) {
    match server_state.boss_bar() {
        BossBar::Enabled {
            title,
            health,
            color,
            division,
        } => {
            let uuid = Uuid::new_v4();
            let (most_sig, least_sig) = uuid.as_u64_pair();
            let packet = BossBarPacket {
                v1_16_uuid: uuid,
                uuid_most_sig: most_sig,
                uuid_least_sig: least_sig,
                action: BossBarAction::Add {
                    title: title.clone(),
                    health: *health,
                    color: (*color).into(),
                    division: BossBarDivisionSerde(*division).0,
                    flags: 0,
                },
            };
            batch.queue(|| PacketRegistry::BossBar(packet));
        }
        BossBar::Disabled => return,
    }
}

fn send_skin_packets(
    batch: &mut Batch<PacketRegistry>,
    client_state: &ClientState,
    server_state: &ServerState,
) {
    let fetch_player_skins = server_state.fetch_player_skins();
    let unique_id = client_state.get_unique_id();
    let protocol_version = client_state.protocol_version();

    // The skin doesn't render before 1.14, probably because there is no world?
    // However, it does render in 1.8, indicated that the packet is well implemented
    // For 1.7.x, it seems like the skin is not sent in this packet
    if protocol_version.is_after_inclusive(ProtocolVersion::V1_8) && !unique_id.is_nil() {
        let username = client_state.get_username();
        let textures = client_state.get_textures();

        batch.queue_async(move || async move {
            let textures: Option<Property> = if textures.is_some() {
                textures
            } else if fetch_player_skins {
                fetch_minecraft_profile(unique_id)
                    .await
                    .ok()
                    .and_then(|profile| profile.try_get_textures())
                    .map(|profile_property| {
                        let textures: Property = profile_property.into();
                        textures
                    })
            } else {
                None
            };

            if let Some(textures) = textures {
                let packet = PlayerInfoUpdatePacket::skin(username, unique_id, textures);
                Some(PacketRegistry::PlayerInfoUpdate(packet))
            } else {
                debug!("No skin for player {username}");
                None
            }
        });
    }

    // There are no skin layers before 1.8 so no need to send this packet
    if protocol_version.is_after_inclusive(ProtocolVersion::V1_8) {
        let packet = SetEntityMetadataPacket::skin_layers(0);
        batch.queue(|| PacketRegistry::SetEntityMetadata(packet));
    }
}

impl From<TryFromIntError> for PacketHandlerError {
    fn from(_: TryFromIntError) -> Self {
        Self::custom("failed to cast int")
    }
}

pub fn send_message(
    batch: &mut Batch<PacketRegistry>,
    component: &Component,
    protocol_version: ProtocolVersion,
) {
    if protocol_version.is_after_inclusive(ProtocolVersion::V1_19) {
        let packet = SystemChatMessagePacket::component(component);
        batch.queue(|| PacketRegistry::SystemChatMessage(packet));
    } else {
        let packet = LegacyChatMessagePacket::component(component);
        batch.queue(|| PacketRegistry::LegacyChatMessage(packet));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::StreamExt;

    fn server_state() -> ServerState {
        let mut builder = ServerState::builder();
        builder.view_distance(0).welcome_message("Hello, World!");
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

    #[tokio::test]
    async fn test_v1_20_3_play_packets() {
        // Given
        let mut client_state = client(ProtocolVersion::V1_20_3);
        let server_state = server_state();
        let mut batch = Batch::new();

        // When
        send_play_packets(&mut batch, &mut client_state, &server_state).unwrap();
        let mut batch = batch.into_stream();

        // Then
        assert!(matches!(
            batch.next().await.unwrap(),
            PacketRegistry::Login(_)
        ));
        assert!(matches!(
            batch.next().await.unwrap(),
            PacketRegistry::SetDefaultSpawnPosition(_)
        ));
        assert!(matches!(
            batch.next().await.unwrap(),
            PacketRegistry::SynchronizePlayerPosition(_)
        ));
        assert!(matches!(
            batch.next().await.unwrap(),
            PacketRegistry::Commands(_)
        ));
        assert!(matches!(
            batch.next().await.unwrap(),
            PacketRegistry::SystemChatMessage(_)
        ));
        assert!(matches!(
            batch.next().await.unwrap(),
            PacketRegistry::UpdateTime(_)
        ));
        assert!(matches!(
            batch.next().await.unwrap(),
            PacketRegistry::SetEntityMetadata(_)
        ));
        assert!(matches!(
            batch.next().await.unwrap(),
            PacketRegistry::GameEvent(_)
        ));
        assert!(matches!(
            batch.next().await.unwrap(),
            PacketRegistry::SetCenterChunk(_)
        ));
        assert!(matches!(
            batch.next().await.unwrap(),
            PacketRegistry::ChunkDataAndUpdateLight(_)
        ));
        assert!(batch.next().await.is_none());
    }

    #[tokio::test]
    async fn test_v1_19_play_packets() {
        // Given
        let mut client_state = client(ProtocolVersion::V1_19);
        let server_state = server_state();
        let mut batch = Batch::new();

        // When
        send_play_packets(&mut batch, &mut client_state, &server_state).unwrap();
        let mut batch = batch.into_stream();

        // Then
        assert!(matches!(
            batch.next().await.unwrap(),
            PacketRegistry::Login(_)
        ));
        assert!(matches!(
            batch.next().await.unwrap(),
            PacketRegistry::SetDefaultSpawnPosition(_)
        ));
        assert!(matches!(
            batch.next().await.unwrap(),
            PacketRegistry::SynchronizePlayerPosition(_)
        ));
        assert!(matches!(
            batch.next().await.unwrap(),
            PacketRegistry::Commands(_)
        ));
        assert!(matches!(
            batch.next().await.unwrap(),
            PacketRegistry::PlayClientBoundPluginMessage(_)
        ));
        assert!(matches!(
            batch.next().await.unwrap(),
            PacketRegistry::SystemChatMessage(_)
        ));
        assert!(matches!(
            batch.next().await.unwrap(),
            PacketRegistry::UpdateTime(_)
        ));
        assert!(matches!(
            batch.next().await.unwrap(),
            PacketRegistry::SetEntityMetadata(_)
        ));
        assert!(matches!(
            batch.next().await.unwrap(),
            PacketRegistry::SetCenterChunk(_)
        ));
        assert!(matches!(
            batch.next().await.unwrap(),
            PacketRegistry::ChunkDataAndUpdateLight(_)
        ));
        assert!(batch.next().await.is_none());
    }

    #[tokio::test]
    async fn test_v1_13_play_packets() {
        // Given
        let mut client_state = client(ProtocolVersion::V1_13);
        let server_state = server_state();
        let mut batch = Batch::new();

        // When
        send_play_packets(&mut batch, &mut client_state, &server_state).unwrap();
        let mut batch = batch.into_stream();

        // Then
        assert!(matches!(
            batch.next().await.unwrap(),
            PacketRegistry::Login(_)
        ));
        assert!(matches!(
            batch.next().await.unwrap(),
            PacketRegistry::SynchronizePlayerPosition(_)
        ));
        assert!(matches!(
            batch.next().await.unwrap(),
            PacketRegistry::Commands(_)
        ));
        assert!(matches!(
            batch.next().await.unwrap(),
            PacketRegistry::PlayClientBoundPluginMessage(_)
        ));
        assert!(matches!(
            batch.next().await.unwrap(),
            PacketRegistry::LegacyChatMessage(_)
        ));
        assert!(matches!(
            batch.next().await.unwrap(),
            PacketRegistry::UpdateTime(_)
        ));
        assert!(matches!(
            batch.next().await.unwrap(),
            PacketRegistry::SetEntityMetadata(_)
        ));
        assert!(batch.next().await.is_none());
    }

    #[tokio::test]
    async fn test_pre_modern_play_packets() {
        // Given
        let mut client_state = client(ProtocolVersion::V1_12_2);
        let server_state = server_state();
        let mut batch = Batch::new();

        // When
        send_play_packets(&mut batch, &mut client_state, &server_state).unwrap();
        let mut batch = batch.into_stream();

        // Then
        assert!(matches!(
            batch.next().await.unwrap(),
            PacketRegistry::Login(_)
        ));
        assert!(matches!(
            batch.next().await.unwrap(),
            PacketRegistry::SynchronizePlayerPosition(_)
        ));
        assert!(matches!(
            batch.next().await.unwrap(),
            PacketRegistry::LegacyChatMessage(_)
        ));
        assert!(matches!(
            batch.next().await.unwrap(),
            PacketRegistry::UpdateTime(_)
        ));
        assert!(matches!(
            batch.next().await.unwrap(),
            PacketRegistry::SetEntityMetadata(_)
        ));
        assert!(batch.next().await.is_none());
    }
}
