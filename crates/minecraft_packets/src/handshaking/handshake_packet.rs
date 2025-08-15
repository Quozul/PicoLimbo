use minecraft_protocol::prelude::*;

#[derive(Debug, Clone, PacketIn)]
pub struct HandshakePacket {
    pub protocol: VarInt,
    pub hostname: String,
    pub port: u16,
    /// 1: Status, 2: Login, 3: Transfer
    pub next_state: VarInt,
}

impl HandshakePacket {
    pub fn status(protocol_version: ProtocolVersion, hostname: String, port: u16) -> Self {
        Self {
            protocol: protocol_version.version_number().into(),
            hostname,
            port,
            next_state: 1.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::handshaking::handshake_packet::HandshakePacket;
    use minecraft_protocol::prelude::{BinaryReader, DecodePacket, ProtocolVersion, VarInt};

    #[test]
    fn test_handshake_packet_decode() {
        let handshake_snapshot = [
            129, 6, 9, 108, 111, 99, 97, 108, 104, 111, 115, 116, 99, 221, 1,
        ];
        let mut reader = BinaryReader::new(&handshake_snapshot);
        let protocol_version = ProtocolVersion::V1_21_4;
        let expected_protocol = VarInt::new(769);
        let expected_hostname = "localhost".to_string();
        let expected_port = 25565;
        let expected_next_state = VarInt::new(1);

        let packet = HandshakePacket::decode(&mut reader, protocol_version).unwrap();

        assert_eq!(expected_protocol, packet.protocol);
        assert_eq!(expected_hostname, packet.hostname);
        assert_eq!(expected_port, packet.port);
        assert_eq!(expected_next_state, packet.next_state);
    }
}
