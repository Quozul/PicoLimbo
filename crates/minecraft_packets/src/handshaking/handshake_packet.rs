use minecraft_protocol::prelude::*;
use minecraft_protocol::protocol_version::ProtocolVersion;

#[derive(Debug, Clone, PacketIn, PacketOut)]
#[packet_id("handshake/serverbound/minecraft:intention")]
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
    use minecraft_protocol::prelude::{
        BinaryReader, BinaryWriter, DecodePacket, EncodePacket, ProtocolVersion, VarInt,
    };

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

    #[test]
    fn test_handshake_packet_encode() {
        let protocol_version = ProtocolVersion::V1_21_4;
        let protocol = VarInt::new(769);
        let hostname = "localhost".to_string();
        let port = 25565;
        let next_state = VarInt::new(1);
        let expected_bytes = vec![
            129, 6, 9, 108, 111, 99, 97, 108, 104, 111, 115, 116, 99, 221, 1,
        ];

        let packet = HandshakePacket {
            protocol,
            hostname,
            port,
            next_state,
        };

        let mut writer = BinaryWriter::new();
        packet.encode(&mut writer, protocol_version).unwrap();
        let encoded = writer.into_inner();

        assert_eq!(expected_bytes, encoded);
    }
}
