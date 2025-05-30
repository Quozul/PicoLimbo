use crate::server_state::ServerState;
use minecraft_packets::configuration::client_bound_known_packs_packet::ClientBoundKnownPacksPacket;
use minecraft_packets::configuration::configuration_client_bound_plugin_message_packet::ConfigurationClientBoundPluginMessagePacket;
use minecraft_packets::configuration::finish_configuration_packet::FinishConfigurationPacket;
use minecraft_packets::configuration::registry_data_packet::{
    RegistryDataCodecPacket, RegistryDataPacket,
};
use minecraft_packets::play::chunk_data_and_update_light_packet::ChunkDataAndUpdateLightPacket;
use minecraft_packets::play::game_event_packet::GameEventPacket;
use minecraft_packets::play::login_packet::LoginPacket;
use minecraft_packets::play::play_client_bound_plugin_message_packet::PlayClientBoundPluginMessagePacket;
use minecraft_packets::play::set_default_spawn_position::SetDefaultSpawnPosition;
use minecraft_packets::play::synchronize_player_position_packet::SynchronizePlayerPositionPacket;
use minecraft_packets::play::system_chat_message_packet::SystemChatMessage;
use minecraft_protocol::data::registry::get_all_registries::{
    get_v1_16_2_registry_codec, get_v1_16_registry_codec, get_v1_20_5_registries,
};
use minecraft_protocol::prelude::Nbt;
use minecraft_protocol::protocol_version::ProtocolVersion;
use minecraft_protocol::state::State;
use minecraft_server::client::Client;
use minecraft_server::server::GetDataDirectory;

/// Only for <= 1.20.2
pub async fn send_configuration_packets(client: Client, state: ServerState) {
    // Send Server Brand
    let packet = ConfigurationClientBoundPluginMessagePacket::brand("PicoLimbo");
    client.send_packet(packet).await;
    let protocol_version = client.protocol_version().await;

    if ProtocolVersion::V1_20_5 <= protocol_version {
        // Send Known Packs
        let packet = ClientBoundKnownPacksPacket::default();
        client.send_packet(packet).await;

        // Send Registry Data
        let grouped = get_v1_20_5_registries(protocol_version, state.data_directory());
        for (registry_id, entries) in grouped {
            let packet = RegistryDataPacket {
                registry_id,
                entries: entries.into(),
            };
            client.send_packet(packet).await;
        }
    } else {
        // Only for 1.20.2 and 1.20.3
        let registry_codec = get_v1_16_2_registry_codec(&protocol_version, state.data_directory());
        let packet = RegistryDataCodecPacket { registry_codec };
        client.send_packet(packet).await;
    }

    // Send Finished Configuration
    let packet = FinishConfigurationPacket {};
    client.send_packet(packet).await;

    send_play_packets(client, state).await;
}

/// Switch to the Play state and send required packets to spawn the player in the world
pub async fn send_play_packets(client: Client, state: ServerState) {
    let protocol_version = client.protocol_version().await;
    let (registry_codec, dimension) = if ProtocolVersion::V1_20_2 <= protocol_version {
        // Since 1.20.2, registries are sent during the configuration state,
        // it is no longer sent in the login packet
        (Nbt::End, Nbt::End)
    } else {
        let registry_codec = if protocol_version == ProtocolVersion::V1_16
            || protocol_version == ProtocolVersion::V1_16_1
        {
            get_v1_16_registry_codec(state.data_directory()).unwrap()
        } else {
            get_v1_16_2_registry_codec(&protocol_version, state.data_directory())
        };

        // For versions between 1.16.2 and 1.18.2 (included), we must send the dimension codec separately
        let dimension = if protocol_version
            .between_inclusive(ProtocolVersion::V1_16_2, ProtocolVersion::V1_18_2)
        {
            let dimension_types = registry_codec
                .find_tag("minecraft:dimension_type")
                .unwrap()
                .find_tag("value")
                .unwrap()
                .get_vec()
                .unwrap();

            let dimension = dimension_types
                .iter()
                .find(|element| {
                    element
                        .find_tag("name".to_string())
                        .map(|name| match name {
                            Nbt::String { value, .. } => {
                                value == &state.spawn_dimension().identifier().to_string()
                            }
                            _ => false,
                        })
                        .unwrap()
                })
                .cloned()
                .unwrap_or(dimension_types[0].clone());

            dimension.find_tag("element").unwrap().clone()
        } else {
            Nbt::End
        };
        (registry_codec, dimension)
    };

    let packet =
        LoginPacket::new_with_dimension(state.spawn_dimension(), registry_codec, dimension);
    client.send_packet(packet).await;

    // Send Synchronize Player Position
    let packet = SynchronizePlayerPositionPacket::default();
    client.send_packet(packet).await;

    if protocol_version >= ProtocolVersion::V1_19 {
        // Send Set Default Spawn Position
        let packet = SetDefaultSpawnPosition::default();
        client.send_packet(packet).await;
    }

    if protocol_version >= ProtocolVersion::V1_20_3 {
        // Send Game Event
        let packet = GameEventPacket::start_waiting_for_chunks(0.0);
        client.send_packet(packet).await;

        // Send Chunk Data and Update Light
        let packet = ChunkDataAndUpdateLightPacket::new(protocol_version.clone());
        client.send_packet(packet).await;
    }

    if protocol_version >= ProtocolVersion::V1_8 {
        client.set_state(State::Play).await;
        client.send_keep_alive().await;
    }

    // The brand is not visible for clients prior to 1.13, no need to send it
    // The brand is sent during the configuration state after 1.20.2 included
    if protocol_version.between_inclusive(ProtocolVersion::V1_13, ProtocolVersion::V1_20) {
        let packet = PlayClientBoundPluginMessagePacket::brand("PicoLimbo");
        client.send_packet(packet).await;
    }

    if let Some(content) = state.welcome_message() {
        if protocol_version >= ProtocolVersion::V1_19 {
            let packet = SystemChatMessage::plain_text(content);
            client.send_packet(packet).await;
        }
    }
}
