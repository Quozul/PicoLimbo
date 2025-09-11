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
    ) -> Result<(), PacketHandlerError> {
        let protocol_version = client_state.protocol_version();
        if protocol_version.supports_configuration_state() {
            client_state.set_state(State::Configuration);
            send_configuration_packets(client_state, server_state);
            Ok(())
        } else {
            Err(PacketHandlerError::invalid_state(
                "Configuration state not supported for this version",
            ))
        }
    }
}

/// Only for <= 1.20.2
fn send_configuration_packets(client_state: &mut ClientState, server_state: &ServerState) {
    // Send Server Brand
    let packet = ConfigurationClientBoundPluginMessagePacket::brand("PicoLimbo");
    client_state.queue_packet(PacketRegistry::ConfigurationClientBoundPluginMessage(
        packet,
    ));
    let protocol_version = client_state.protocol_version();

    if protocol_version.is_after_inclusive(ProtocolVersion::V1_20_5) {
        // Send Known Packs
        let packet = ClientBoundKnownPacksPacket::new(protocol_version.humanize());
        client_state.queue_packet(PacketRegistry::ClientBoundKnownPacks(packet));
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
                client_state.queue_packet(PacketRegistry::RegistryData(packet));
            }
        }
        Registries::V1_20_2 { registry_codec } => {
            let packet = RegistryDataPacket::codec(registry_codec);
            client_state.queue_packet(PacketRegistry::RegistryData(packet));
        }
        _ => {
            unreachable!()
        }
    }

    // Send Finished Configuration
    let packet = FinishConfigurationPacket {};
    client_state.queue_packet(PacketRegistry::FinishConfiguration(packet));
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
        pkt.handle(&mut client_state, &server_state).unwrap();

        // Then
        assert_eq!(client_state.state(), State::Configuration);
        assert!(!client_state.has_no_more_packets());
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
        assert!(client_state.has_no_more_packets());
    }

    #[test]
    fn test_configuration_packets_v1_20_2() {
        // Given
        let mut client_state = client(ProtocolVersion::V1_20_2);
        let server_state = server_state();

        // When
        send_configuration_packets(&mut client_state, &server_state);

        // Then
        assert!(matches!(
            client_state.next_packet(),
            PacketRegistry::ConfigurationClientBoundPluginMessage(_)
        ));
        assert!(matches!(
            client_state.next_packet(),
            PacketRegistry::RegistryData(_)
        ));
        assert!(matches!(
            client_state.next_packet(),
            PacketRegistry::FinishConfiguration(_)
        ));
        assert!(client_state.has_no_more_packets());
    }

    #[test]
    fn test_configuration_packets_v1_20_5() {
        // Given
        let mut client_state = client(ProtocolVersion::V1_20_5);
        let server_state = server_state();

        // When
        send_configuration_packets(&mut client_state, &server_state);

        // Then
        assert!(matches!(
            client_state.next_packet(),
            PacketRegistry::ConfigurationClientBoundPluginMessage(_)
        ));
        assert!(matches!(
            client_state.next_packet(),
            PacketRegistry::ClientBoundKnownPacks(_)
        ));
        for _ in 0..4 {
            assert!(matches!(
                client_state.next_packet(),
                PacketRegistry::RegistryData(_)
            ));
        }
        assert!(matches!(
            client_state.next_packet(),
            PacketRegistry::FinishConfiguration(_)
        ));
        assert!(client_state.has_no_more_packets());
    }
}
