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
use pico_precomputed_registries::PrecomputedRegistries;
use pico_registries::registry_provider::RegistryProvider;

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
            send_configuration_packets(&mut batch, protocol_version)?;
            Ok(batch)
        } else {
            Err(PacketHandlerError::invalid_state(
                "Configuration state not supported for this version",
            ))
        }
    }
}

/// Only for >= 1.20.2
fn send_configuration_packets(
    batch: &mut Batch<PacketRegistry>,
    protocol_version: ProtocolVersion,
) -> Result<(), PacketHandlerError> {
    let registry_provider = PrecomputedRegistries::new(protocol_version);

    // Send Server Brand
    let packet = ConfigurationClientBoundPluginMessagePacket::brand("PicoLimbo");
    batch.queue(|| PacketRegistry::ConfigurationClientBoundPluginMessage(packet));

    if protocol_version.is_after_inclusive(ProtocolVersion::V1_20_5) {
        // Send Known Packs
        let packet = ClientBoundKnownPacksPacket::new(protocol_version.humanize());
        batch.queue(|| PacketRegistry::ClientBoundKnownPacks(packet));
    }

    // Send tags
    if protocol_version.is_after_inclusive(ProtocolVersion::V1_21_11) {
        // Since 1.21.11, only the Timeline tags are required
        // All tags are sent in a single packet
        // TODO: `wolf_variant` tags should probably be sent too
        let tagged_registries = registry_provider
            .get_tagged_registries()?
            .iter()
            .map(|tagged_registry| {
                TaggedRegistry::new(
                    tagged_registry.registry_id.clone(),
                    tagged_registry
                        .tags
                        .iter()
                        .map(|registry_tag| {
                            RegistryTag::new(
                                registry_tag.identifier.clone(),
                                registry_tag.ids.iter().map(VarInt::from).collect(),
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
        // Since 1.20.5, each registry is sent in its own packet
        batch.chain_iter(
            registry_provider
                .get_registry_data_v1_20_5()?
                .into_iter()
                .map(|(registry_id, registry_entries)| {
                    let packet = RegistryDataPacket::registry(
                        registry_id,
                        registry_entries
                            .iter()
                            .map(|entry| {
                                RegistryEntry::new(entry.entry_id.clone(), entry.nbt_bytes.clone())
                            })
                            .collect(),
                    );
                    PacketRegistry::RegistryData(packet)
                }),
        );
    } else if protocol_version.is_after_inclusive(ProtocolVersion::V1_20_2) {
        // Since 1.19, all registries are sent as a single NBT tag
        // Since 1.20.2, all registries are sent in their own packet during the configuration state, still as a single NBT tag
        let registry_codec = registry_provider.get_registry_codec_v1_16()?;
        let packet = RegistryDataPacket::codec(registry_codec);
        batch.queue(|| PacketRegistry::RegistryData(packet));
    } else {
        // Registries are sent in the Join Game packet for versions prior to 1.20.2 since configuration state does not exist
        unreachable!();
    }

    // Send Finished Configuration
    let packet = FinishConfigurationPacket {};
    batch.queue(|| PacketRegistry::FinishConfiguration(packet));
    Ok(())
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
        send_configuration_packets(&mut batch, ProtocolVersion::V1_20_2).unwrap();
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
        send_configuration_packets(&mut batch, ProtocolVersion::V1_20_5).unwrap();
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
