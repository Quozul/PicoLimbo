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
use minecraft_packets::login::login_acknowledged_packet::LoginAcknowledgedPacket;
use minecraft_protocol::prelude::{ProtocolVersion, State};
use registries::{Registries, get_registries};

impl PacketHandler for LoginAcknowledgedPacket {
    fn handle(
        &self,
        client_state: &mut ClientState,
        server_state: &ServerState,
    ) -> Result<Batch<PacketRegistry>, PacketHandlerError> {
        let mut batch = Batch::new();
        let protocol_version = client_state.protocol_version();
        if protocol_version.supports_configuration_state() {
            client_state.set_state(State::Configuration);
            send_configuration_packets(&mut batch, protocol_version, server_state);
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
    server_state: &ServerState,
) {
    // Send Server Brand
    let packet = ConfigurationClientBoundPluginMessagePacket::brand("PicoLimbo");
    batch.queue(|| PacketRegistry::ConfigurationClientBoundPluginMessage(packet));

    if protocol_version.is_after_inclusive(ProtocolVersion::V1_20_5) {
        // Send Known Packs
        let packet = ClientBoundKnownPacksPacket::new(protocol_version.humanize());
        batch.queue(|| PacketRegistry::ClientBoundKnownPacks(packet));
    }

    // Send Registry Data
    match get_registries(protocol_version, server_state.spawn_dimension()) {
        Registries::V1_20_5 { registries } => {
            for registries in registries.registries.into_inner() {
                let entries = registries.entries.into_inner();
                let mut registry_entries = Vec::with_capacity(entries.len());

                for entry in entries {
                    let bytes = entry.nbt_bytes.into_inner();
                    let entry = RegistryEntry::new(entry.entry_id.clone(), bytes);
                    registry_entries.push(entry);
                }

                let packet = RegistryDataPacket::registry(registries.registry_id, registry_entries);
                batch.queue(|| PacketRegistry::RegistryData(packet));
            }
        }
        Registries::V1_20_2 { registry_codec } => {
            let packet = RegistryDataPacket::codec(registry_codec);
            batch.queue(|| PacketRegistry::RegistryData(packet));
        }
        _ => {
            unreachable!()
        }
    }

    // Send Finished Configuration
    let packet = FinishConfigurationPacket {};
    batch.queue(|| PacketRegistry::FinishConfiguration(packet));
}

#[cfg(test)]
mod tests {
    use super::*;
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

    #[test]
    fn test_login_ack_supported_protocol() {
        // Given
        let mut client_state = client(ProtocolVersion::V1_20_2);
        let server_state = server_state();
        let pkt = packet();

        // When
        let batch = pkt.handle(&mut client_state, &server_state).unwrap();
        let mut batch = batch.into_iter();

        // Then
        assert_eq!(client_state.state(), State::Configuration);
        assert!(batch.next().is_some());
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

    #[test]
    fn test_configuration_packets_v1_20_2() {
        // Given
        let server_state = server_state();
        let mut batch = Batch::new();

        // When
        send_configuration_packets(&mut batch, ProtocolVersion::V1_20_2, &server_state);
        let mut batch = batch.into_iter();

        // Then
        assert!(matches!(
            batch.next().unwrap(),
            PacketRegistry::ConfigurationClientBoundPluginMessage(_)
        ));
        assert!(matches!(
            batch.next().unwrap(),
            PacketRegistry::RegistryData(_)
        ));
        assert!(matches!(
            batch.next().unwrap(),
            PacketRegistry::FinishConfiguration(_)
        ));
        assert!(batch.next().is_none());
    }

    #[test]
    fn test_configuration_packets_v1_20_5() {
        // Given
        let server_state = server_state();
        let mut batch = Batch::new();

        // When
        send_configuration_packets(&mut batch, ProtocolVersion::V1_20_5, &server_state);
        let mut batch = batch.into_iter();

        // Then
        assert!(matches!(
            batch.next().unwrap(),
            PacketRegistry::ConfigurationClientBoundPluginMessage(_)
        ));
        assert!(matches!(
            batch.next().unwrap(),
            PacketRegistry::ClientBoundKnownPacks(_)
        ));
        for _ in 0..4 {
            assert!(matches!(
                batch.next().unwrap(),
                PacketRegistry::RegistryData(_)
            ));
        }
        assert!(matches!(
            batch.next().unwrap(),
            PacketRegistry::FinishConfiguration(_)
        ));
        assert!(batch.next().is_none());
    }
}
