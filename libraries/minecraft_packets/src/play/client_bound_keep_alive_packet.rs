use data_types::prelude::EncodePacketField;
use macros::PacketOut;
use protocol_version::ProtocolVersion;
use protocol_version::ProtocolVersion::{V1_7_2, V1_21_4};

#[derive(Debug, PacketOut)]
pub struct ClientBoundKeepAlivePacket {
    id: i64,
}

impl ClientBoundKeepAlivePacket {
    pub fn new(id: i64) -> Self {
        Self { id }
    }
}

impl PacketId for ClientBoundKeepAlivePacket {
    fn packet_id(protocol_version: &ProtocolVersion) -> Option<u8> {
        match protocol_version {
            V1_21_4 => Some(0x27),
            s if s >= &V1_7_2 && s < &V1_21_4 => Some(0x1F),
            _ => None,
        }
    }
}
