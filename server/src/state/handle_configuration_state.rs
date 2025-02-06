use crate::packet_error::PacketError;
use protocol::prelude::configuration::acknowledge_finish_configuration_packet::AcknowledgeConfigurationPacket;
use protocol::prelude::configuration::client_known_packs_packet::ClientKnownPacksPacket;
use protocol::prelude::configuration::server_bound_information_packet::ServerBoundInformationPacket;
use protocol::prelude::configuration::server_bound_plugin_message_packet::ServerBoundPluginMessagePacket;
use protocol::prelude::handshaking::data::state::State;
use protocol::prelude::{DecodePacket, PacketId, ProtocolVersion};

pub enum ConfigurationResult {
    SendConfiguration,
    Play,
    Nothing,
}

pub fn handle_configuration_state(
    packet_id: u8,
    payload: &[u8],
    protocol_version: &ProtocolVersion,
) -> Result<ConfigurationResult, PacketError> {
    if ServerBoundPluginMessagePacket::is_packet(packet_id, protocol_version) {
        ServerBoundPluginMessagePacket::decode(payload, protocol_version)?;
        Ok(ConfigurationResult::SendConfiguration)
    } else if ServerBoundInformationPacket::is_packet(packet_id, protocol_version) {
        ServerBoundInformationPacket::decode(payload, protocol_version)?;
        Ok(ConfigurationResult::Nothing)
    } else if ClientKnownPacksPacket::is_packet(packet_id, protocol_version) {
        ClientKnownPacksPacket::decode(payload, protocol_version)?;
        Ok(ConfigurationResult::Nothing)
    } else if AcknowledgeConfigurationPacket::is_packet(packet_id, protocol_version) {
        AcknowledgeConfigurationPacket::decode(payload, protocol_version)?;
        Ok(ConfigurationResult::Play)
    } else {
        Err(PacketError::new(State::Configuration, packet_id))
    }
}
