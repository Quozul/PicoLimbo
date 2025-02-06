pub trait PacketId {
    fn packet_id(protocol_version: &protocol_version::ProtocolVersion) -> Option<u8>;

    fn get_packet_id(&self, protocol_version: &protocol_version::ProtocolVersion) -> Option<u8> {
        Self::packet_id(protocol_version)
    }

    fn is_packet(packet_id: u8, protocol_version: &protocol_version::ProtocolVersion) -> bool {
        Self::packet_id(protocol_version) == Some(packet_id)
    }

    fn can_send_packet(&self, protocol_version: &protocol_version::ProtocolVersion) -> bool {
        Self::packet_id(protocol_version) != None
    }
}
