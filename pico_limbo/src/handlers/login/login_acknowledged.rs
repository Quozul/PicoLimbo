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

/// Returns the exact Mojang pack version string for the `minecraft:core` known
/// pack that the client expects for a given protocol version. The client compares
/// this string strictly; sending a less-specific version (e.g. `"26.1"` for a
/// client running `26.1.2`) causes the client to reject the pack and respond with
/// an empty `select_known_packs` list, forcing the server to send every registry
/// instead of letting the client use its bundled vanilla data.
///
/// `ProtocolVersion::humanize()` returns `"26.1"` for `V26_1` because that's the
/// minor name; the actual Mojang release tag is `"26.1.2"`. Any protocol version
/// whose release tag differs from `humanize()` needs an arm here.
fn known_pack_version(version: ProtocolVersion) -> &'static str {
    match version {
        ProtocolVersion::V26_1 => "26.1.2",
        _ => version.humanize(),
    }
}

/// Sends the first phase of the configuration burst for protocols `>= 1.20.2`.
///
/// For `>= 1.20.5` this sends only brand + `select_known_packs` and stops, leaving
/// the rest of the burst (tags, registry data, `FinishConfiguration`) to
/// `ServerBoundKnownPacksPacket::handle`, which is invoked when the client
/// responds with its accepted packs. This mirrors Mojang vanilla / Paper: we
/// don't dump every registry if the client already has them locally, and we
/// don't transition the client to PLAY prematurely.
///
/// For `1.20.2`–`1.20.4` (no known-packs handshake), we send the full burst in
/// one go since the protocol can't negotiate.
fn send_configuration_packets(
    batch: &mut Batch<PacketRegistry>,
    protocol_version: ProtocolVersion,
) -> Result<(), PacketHandlerError> {
    // Send Server Brand
    let packet = ConfigurationClientBoundPluginMessagePacket::brand(SERVER_BRAND);
    batch.queue(|| PacketRegistry::ConfigurationClientBoundPluginMessage(packet));

    if protocol_version.is_after_inclusive(ProtocolVersion::V1_20_5) {
        // Send Known Packs and stop. The rest is sent by
        // ServerBoundKnownPacksPacket::handle once the client responds.
        let packet = ClientBoundKnownPacksPacket::new(known_pack_version(protocol_version));
        batch.queue(|| PacketRegistry::ClientBoundKnownPacks(packet));
    } else {
        // Pre-1.20.5: no handshake, send everything now.
        send_post_known_packs_configuration_packets(batch, protocol_version, false)?;
    }

    Ok(())
}

/// Sends the configuration packets that follow the optional KnownPacks handshake:
/// tags (`>= 1.21.6`), registry data (unless the client accepted vanilla), and
/// `FinishConfiguration`.
///
/// `client_accepted_vanilla_core` indicates whether the client accepted the
/// `minecraft:core` known pack we offered. When true, the client already has all
/// vanilla registry data locally and we skip sending it — matching Mojang
/// vanilla / Paper behavior. When false (or pre-1.20.5), we send every registry.
fn send_post_known_packs_configuration_packets(
    batch: &mut Batch<PacketRegistry>,
    protocol_version: ProtocolVersion,
    client_accepted_vanilla_core: bool,
) -> Result<(), PacketHandlerError> {
    let registry_provider = PrecomputedRegistries::new(protocol_version);

    // Send tags
    if protocol_version.is_after_inclusive(ProtocolVersion::V1_21_6) {
        // Since 1.21.6, the Dialog tags should be sent to have server links working.
        // Since 1.21.11, the Timeline tags should be sent to get the time of day working.
        // All tags are sent in a single packet.
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

    // Send Registry Data — only if the client did not accept the vanilla pack.
    if !client_accepted_vanilla_core {
        if protocol_version.is_after_inclusive(ProtocolVersion::V1_20_5) {
            // Since 1.20.5, each registry is sent in its own packet.
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
                                    RegistryEntry::new(
                                        entry.entry_id.clone(),
                                        entry.nbt_bytes.clone(),
                                    )
                                })
                                .collect(),
                        );
                        PacketRegistry::RegistryData(packet)
                    }),
            );
        } else if protocol_version.is_after_inclusive(ProtocolVersion::V1_20_2) {
            // Since 1.20.2 (no per-registry packets yet) the codec is sent as a
            // single NBT compound.
            let registry_codec = registry_provider.get_registry_codec_v1_16()?;
            let packet = RegistryDataPacket::codec(registry_codec);
            batch.queue(|| PacketRegistry::RegistryData(packet));
        } else {
            // Registries are sent in the Join Game packet for versions prior to
            // 1.20.2 since configuration state does not exist.
            unreachable!();
        }
    }

    // Send Finished Configuration
    let packet = FinishConfigurationPacket {};
    batch.queue(|| PacketRegistry::FinishConfiguration(packet));
    Ok(())
}

impl PacketHandler for ServerBoundKnownPacksPacket {
    fn handle(
        &self,
        client_state: &mut ClientState,
        _server_state: &ServerState,
    ) -> Result<Batch<PacketRegistry>, PacketHandlerError> {
        let mut batch = Batch::new();
        let protocol_version = client_state.protocol_version();

        // The client replied with the list of known packs it accepts. If the
        // list contains the vanilla `minecraft:core` pack at the version we
        // proposed, the client has bundled vanilla registry data locally and
        // we can skip sending registry data — matching Mojang vanilla / Paper.
        let expected_version = known_pack_version(protocol_version);
        let client_accepted_vanilla_core = self.known_packs.inner().iter().any(|pack| {
            pack.namespace == "minecraft"
                && pack.id == "core"
                && pack.version == expected_version
        });

        send_post_known_packs_configuration_packets(
            &mut batch,
            protocol_version,
            client_accepted_vanilla_core,
        )?;
        Ok(batch)
    }
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

        // Then: the initial burst only sends brand + KnownPacks. The rest of
        // the configuration (registries + FinishConfiguration) is sent only
        // after the client's `select_known_packs` response.
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
        // Given: client did NOT accept the vanilla pack.
        let mut batch = Batch::new();

        // When
        send_post_known_packs_configuration_packets(
            &mut batch,
            ProtocolVersion::V1_20_5,
            false,
        )
        .unwrap();
        let mut batch = batch.into_stream();

        // Then: registries are sent, followed by FinishConfiguration.
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
    async fn test_post_known_packs_when_vanilla_accepted_skips_registries() {
        // Given: client accepted the vanilla pack — server skips registry data.
        let mut batch = Batch::new();

        // When
        send_post_known_packs_configuration_packets(
            &mut batch,
            ProtocolVersion::V1_20_5,
            true,
        )
        .unwrap();
        let mut batch = batch.into_stream();

        // Then: only FinishConfiguration is sent (no registries).
        assert!(matches!(
            batch.next().await.unwrap(),
            PacketRegistry::FinishConfiguration(_)
        ));
        assert!(batch.next().await.is_none());
    }
}
