use crate::server::batch::Batch;
use crate::server::client_state::ClientState;
use crate::server::packet_handler::{PacketHandler, PacketHandlerError};
use crate::server::packet_registry::PacketRegistry;
use crate::server_brand::SERVER_BRAND;
use crate::server_state::ServerState;
use minecraft_packets::configuration::client_bound_known_packs_packet::ClientBoundKnownPacksPacket;
use minecraft_packets::configuration::configuration_client_bound_plugin_message_packet::ConfigurationClientBoundPluginMessagePacket;
use minecraft_packets::configuration::data::registry_entry::RegistryEntry;
use minecraft_packets::configuration::finish_configuration_packet::FinishConfigurationPacket;
use minecraft_packets::configuration::registry_data_packet::RegistryDataPacket;
use minecraft_packets::configuration::server_bound_known_packs_packet::ServerBoundKnownPacksPacket;
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

impl PacketHandler for ServerBoundKnownPacksPacket {
    fn handle(
        &self,
        client_state: &mut ClientState,
        _server_state: &ServerState,
    ) -> Result<Batch<PacketRegistry>, PacketHandlerError> {
        let mut batch = Batch::new();
        let protocol_version = client_state.protocol_version();
        let client_accepted_vanilla_core = self.has_minecraft_core();
        send_post_known_packs_configuration_packets(
            &mut batch,
            protocol_version,
            client_accepted_vanilla_core,
        )?;
        Ok(batch)
    }
}

/// Only for >= 1.20.2
fn send_configuration_packets(
    batch: &mut Batch<PacketRegistry>,
    protocol_version: ProtocolVersion,
) -> Result<(), PacketHandlerError> {
    // Send Server Brand
    let packet = ConfigurationClientBoundPluginMessagePacket::brand(SERVER_BRAND);
    batch.queue(|| PacketRegistry::ConfigurationClientBoundPluginMessage(packet));

    if protocol_version.is_after_inclusive(ProtocolVersion::V1_20_5) {
        // Send Known Packs and wait for the client's response before sending the rest.
        // The remaining packets (tags, registries, finish) are emitted from the
        // `ServerBoundKnownPacksPacket` handler once we know whether the client
        // accepted the vanilla `minecraft:core` pack we offered.
        let known_packs = protocol_version.known_packs();
        let packet = ClientBoundKnownPacksPacket::new(known_packs);
        batch.queue(|| PacketRegistry::ClientBoundKnownPacks(packet));
    } else {
        send_post_known_packs_configuration_packets(batch, protocol_version, false)?;
    }
    Ok(())
}

fn send_post_known_packs_configuration_packets(
    batch: &mut Batch<PacketRegistry>,
    protocol_version: ProtocolVersion,
    client_accepted_vanilla_core: bool,
) -> Result<(), PacketHandlerError> {
    let registry_provider = PrecomputedRegistries::new(protocol_version);

    // Send Registry Data — skip when the client accepted our vanilla `minecraft:core`
    // offer, since vanilla clients (and Paper-based servers) treat that as a signal
    // that the registries are already known.
    if protocol_version.is_after_inclusive(ProtocolVersion::V1_20_5) {
        let omit_registry_data = protocol_version.is_after_inclusive(ProtocolVersion::V1_21_5)
            && client_accepted_vanilla_core;

        // Since 1.20.5, each registry is sent in its own packet
        batch.chain_iter(
            registry_provider
                .get_registry_data_v1_20_5()?
                .into_iter()
                .map(move |(registry_id, registry_entries)| {
                    let packet = RegistryDataPacket::registry(
                        registry_id,
                        registry_entries
                            .iter()
                            .map(|entry| {
                                let nbt_bytes = if omit_registry_data {
                                    None
                                } else {
                                    Some(entry.nbt_bytes.clone())
                                };
                                RegistryEntry::new(entry.entry_id.clone(), nbt_bytes)
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

    // Send tags
    if protocol_version.is_after_inclusive(ProtocolVersion::V1_21_6) {
        // Since 1.21.6, the Dialog tags should be sent to have server links working
        // Since 1.21.11, the Timeline tags should be sent to get the time of day working
        // All tags are sent in a single packet
        // TODO: `wolf_variant` tags should probably be sent too?
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
        assert!(matches!(
            result,
            Err(PacketHandlerError::InvalidState(_, _))
        ));
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
    async fn test_configuration_packets_v1_20_5_initial_sends_only_brand_and_known_packs() {
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
        assert!(batch.next().await.is_none());
    }

    #[tokio::test]
    async fn test_post_known_packs_when_vanilla_rejected_sends_registries() {
        // Given
        let mut batch = Batch::new();

        // When
        send_post_known_packs_configuration_packets(&mut batch, ProtocolVersion::V1_20_5, false)
            .unwrap();
        let mut batch = batch.into_stream();

        // Then
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

    #[tokio::test]
    async fn test_known_packs_handler_with_empty_list_sends_registries() {
        // Given
        let mut client_state = client(ProtocolVersion::V1_20_5);
        client_state.set_state(State::Configuration);
        let server_state = server_state();
        let pkt = ServerBoundKnownPacksPacket::new(Vec::new());

        // When
        let batch = pkt.handle(&mut client_state, &server_state).unwrap();
        let mut batch = batch.into_stream();

        // Then — registries first, then FinishConfiguration
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
