use crate::identifier_utils::to_protocol_identifier;
use crate::registries_utils::load_registry_manager;
use crate::server::batch::Batch;
use crate::server::client_state::ClientState;
use crate::server::packet_handler::{PacketHandler, PacketHandlerError};
use crate::server::packet_registry::PacketRegistry;
use crate::server_state::ServerState;
use minecraft_packets::configuration::client_bound_known_packs_packet::ClientBoundKnownPacksPacket;
use minecraft_packets::configuration::configuration_client_bound_plugin_message_packet::ConfigurationClientBoundPluginMessagePacket;
use minecraft_packets::configuration::data::registry_entry::RegistryEntry;
use minecraft_packets::configuration::finish_configuration_packet::FinishConfigurationPacket;
use minecraft_packets::configuration::registry_data_packet::RegistryDataPacket;
use minecraft_packets::configuration::update_tags_packet::{
    RegistryTag, TaggedRegistry, UpdateTagsPacket,
};
use minecraft_packets::login::login_acknowledged_packet::LoginAcknowledgedPacket;
use minecraft_protocol::prelude::{ProtocolVersion, State, VarInt};
use pico_nbt2::{CompressionType, NbtOptions};
use pico_registries::{Registry, RegistryKeys, RegistryManagerBuilder};

impl PacketHandler for LoginAcknowledgedPacket {
    fn handle(
        &self,
        client_state: &mut ClientState,
        _server_state: &ServerState,
    ) -> Result<Batch<PacketRegistry>, PacketHandlerError> {
        let mut batch = Batch::new();
        let protocol_version = client_state.protocol_version();
        if protocol_version.supports_configuration_state() {
            client_state.set_state(State::Configuration);
            send_configuration_packets(&mut batch, protocol_version);
            Ok(batch)
        } else {
            Err(PacketHandlerError::invalid_state(
                "Configuration state not supported for this version",
            ))
        }
    }
}

/// Only for <= 1.20.2
fn send_configuration_packets(
    batch: &mut Batch<PacketRegistry>,
    protocol_version: ProtocolVersion,
) {
    // Send Server Brand
    let packet = ConfigurationClientBoundPluginMessagePacket::brand("PicoLimbo");
    batch.queue(|| PacketRegistry::ConfigurationClientBoundPluginMessage(packet));

    if protocol_version.is_after_inclusive(ProtocolVersion::V1_20_5) {
        // Send Known Packs
        let packet = ClientBoundKnownPacksPacket::new(protocol_version.humanize());
        batch.queue(|| PacketRegistry::ClientBoundKnownPacks(packet));
    }

    // Send tags
    if protocol_version.is_after_inclusive(ProtocolVersion::V1_20_5) {
        // For now, only the Timeline tags are required
        let tag_registries = &[RegistryKeys::Timeline];
        let registry_manager = load_registry_manager(protocol_version, tag_registries);

        let tagged_registries = tag_registries
            .iter()
            .map(|registry_keys| {
                let registry = registry_manager
                    .get_optional(registry_keys)
                    .expect("Registry not found");

                let tags = registry.get_tag_identifiers();
                let registry_identifier = registry.get_registry_key().get_value();

                TaggedRegistry::new(
                    to_protocol_identifier(registry_identifier),
                    tags.iter()
                        .map(|tag_name| {
                            RegistryTag::new(
                                to_protocol_identifier(&tag_name.normalize()),
                                evaluate_tags(registry, tag_name),
                            )
                        })
                        .collect(),
                )
            })
            .collect();

        let packet = UpdateTagsPacket::new(tagged_registries);
        batch.queue(|| PacketRegistry::UpdateTags(packet));
    }

    // Send Registry Data
    if protocol_version.is_after_inclusive(ProtocolVersion::V1_20_5) {
        let registries = RegistryManagerBuilder::DEFAULT_REGISTRIES;
        let registry_manager = load_registry_manager(protocol_version, registries);

        for registry_keys in registries {
            let registry = registry_manager
                .get_optional(registry_keys)
                .expect("Registry not found");
            let entries = registry.get_entries();
            let mut registry_entries = Vec::with_capacity(entries.len());

            for entry in entries {
                let bytes = entry
                    .get_raw_value()
                    .to_byte(
                        CompressionType::None,
                        NbtOptions::new().nameless_root(true).dynamic_lists(true),
                        None,
                    )
                    .expect("Failed to serialize registry entry");
                let entry_id = entry.get_registry_key().get_value();
                let entry = RegistryEntry::new(to_protocol_identifier(entry_id), bytes);
                registry_entries.push(entry);
            }

            let registry_id = registry.get_registry_key().get_value();
            let packet =
                RegistryDataPacket::registry(to_protocol_identifier(registry_id), registry_entries);
            batch.queue(|| PacketRegistry::RegistryData(packet));
        }
    } else if protocol_version.is_after_inclusive(ProtocolVersion::V1_20_2) {
        // TODO: Implement registries using new crate
        unimplemented!();
    } else {
        // Registries are sent in the Join Game packet for versions prior to 1.20.2 since configuration state does not exist
        unreachable!();
    }

    // Send Finished Configuration
    let packet = FinishConfigurationPacket {};
    batch.queue(|| PacketRegistry::FinishConfiguration(packet));
}

// This function is called recursively
fn evaluate_tags(registry: &Registry, tag_name: &pico_registries::Identifier) -> Vec<VarInt> {
    registry
        .get_tag(tag_name)
        .expect("Failed to get tag")
        .get_values()
        .iter()
        .flat_map(|identifier| {
            if identifier.is_tag() {
                // If it is a tag, we should expend all the values from that tag into the current tag
                evaluate_tags(registry, &identifier.normalize())
            } else {
                // If it is not a tag, then we should get the protocol ID of the actual value from the registry
                registry
                    .get_optional(identifier)
                    .into_iter()
                    .map(|entry| VarInt::from(entry.get_protocol_id()))
                    .collect()
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::StreamExt;
    use minecraft_protocol::prelude::ProtocolVersion;

    fn server_state() -> ServerState {
        ServerState::builder().build().unwrap()
    }

    fn client(protocol: ProtocolVersion) -> ClientState {
        let mut cs = ClientState::default();
        cs.set_protocol_version(protocol);
        cs.set_state(State::Login);
        cs
    }

    fn packet() -> LoginAcknowledgedPacket {
        LoginAcknowledgedPacket::default()
    }

    #[tokio::test]
    async fn test_login_ack_supported_protocol() {
        // Given
        let mut client_state = client(ProtocolVersion::V1_20_2);
        let server_state = server_state();
        let pkt = packet();

        // When
        let batch = pkt.handle(&mut client_state, &server_state).unwrap();
        let mut batch = batch.into_stream();

        // Then
        assert_eq!(client_state.state(), State::Configuration);
        assert!(batch.next().await.is_some());
    }

    #[test]
    fn test_login_ack_unsupported_protocol() {
        // Given
        let mut client_state = client(ProtocolVersion::V1_20);
        let server_state = server_state();
        let pkt = packet();

        // When
        let result = pkt.handle(&mut client_state, &server_state);

        // Then
        assert!(matches!(result, Err(PacketHandlerError::InvalidState(_))));
    }

    #[tokio::test]
    async fn test_configuration_packets_v1_20_2() {
        // Given
        let mut batch = Batch::new();

        // When
        send_configuration_packets(&mut batch, ProtocolVersion::V1_20_2);
        let mut batch = batch.into_stream();

        // Then
        assert!(matches!(
            batch.next().await.unwrap(),
            PacketRegistry::ConfigurationClientBoundPluginMessage(_)
        ));
        assert!(matches!(
            batch.next().await.unwrap(),
            PacketRegistry::RegistryData(_)
        ));
        assert!(matches!(
            batch.next().await.unwrap(),
            PacketRegistry::FinishConfiguration(_)
        ));
        assert!(batch.next().await.is_none());
    }

    #[tokio::test]
    async fn test_configuration_packets_v1_20_5() {
        // Given
        let mut batch = Batch::new();

        // When
        send_configuration_packets(&mut batch, ProtocolVersion::V1_20_5);
        let mut batch = batch.into_stream();

        // Then
        assert!(matches!(
            batch.next().await.unwrap(),
            PacketRegistry::ConfigurationClientBoundPluginMessage(_)
        ));
        assert!(matches!(
            batch.next().await.unwrap(),
            PacketRegistry::ClientBoundKnownPacks(_)
        ));
        for _ in 0..4 {
            assert!(matches!(
                batch.next().await.unwrap(),
                PacketRegistry::RegistryData(_)
            ));
        }
        assert!(matches!(
            batch.next().await.unwrap(),
            PacketRegistry::FinishConfiguration(_)
        ));
        assert!(batch.next().await.is_none());
    }
}
