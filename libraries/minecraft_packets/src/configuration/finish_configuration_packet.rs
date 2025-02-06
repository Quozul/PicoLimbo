use macros::PacketOut;
use protocol_version::ProtocolVersion;
use protocol_version::ProtocolVersion::{V1_7_2, V1_21_4};

#[derive(Debug, PacketOut)]
pub struct FinishConfigurationPacket {}

impl PacketId for FinishConfigurationPacket {
    fn packet_id(protocol_version: &ProtocolVersion) -> Option<u8> {
        match protocol_version {
            V1_21_4 => Some(0x03),
            s if s >= &V1_7_2 && s < &V1_21_4 => Some(0x02),
            _ => None,
        }
    }
}
