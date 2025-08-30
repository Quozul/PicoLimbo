use minecraft_protocol::prelude::*;
use rand::Rng;
use std::num::TryFromIntError;

/// This packet exists for all versions of the game from 1.7.2 to the latest at the time (1.21.4).
#[derive(PacketOut)]
pub struct ClientBoundKeepAlivePacket {
    #[pvn(340..)]
    v1_12_2_id: i64,
    #[pvn(47..340)]
    v1_8_id: VarInt,
    #[pvn(..47)]
    id: i32,
}

impl ClientBoundKeepAlivePacket {
    pub fn new(id: i32) -> Result<Self, TryFromIntError> {
        Ok(Self {
            v1_12_2_id: id.into(),
            v1_8_id: id.into(),
            id,
        })
    }

    pub fn random() -> Result<Self, TryFromIntError> {
        Self::new(get_random_i32())
    }
}

fn get_random_i32() -> i32 {
    let mut rng = rand::rng();
    rng.random()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keep_alive_packet_v1_12_2() {
        let packet = ClientBoundKeepAlivePacket::new(0).unwrap();
        let mut writer = BinaryWriter::new();
        packet
            .encode(&mut writer, ProtocolVersion::V1_12_2)
            .unwrap();
        let encoded_packet = writer.into_inner();
        assert_eq!(
            encoded_packet,
            vec![0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
        );
    }

    #[test]
    fn test_keep_alive_packet_v1_8() {
        let packet = ClientBoundKeepAlivePacket::new(0).unwrap();
        let mut writer = BinaryWriter::new();
        packet.encode(&mut writer, ProtocolVersion::V1_8).unwrap();
        let encoded_packet = writer.into_inner();
        assert_eq!(encoded_packet, vec![0x00]);
    }

    #[test]
    fn test_keep_alive_packet_v1_7_2() {
        let packet = ClientBoundKeepAlivePacket::new(0).unwrap();
        let mut writer = BinaryWriter::new();
        packet.encode(&mut writer, ProtocolVersion::V1_7_2).unwrap();
        let encoded_packet = writer.into_inner();
        assert_eq!(encoded_packet, vec![0x00, 0x00, 0x00, 0x00]);
    }
}
