use data_types::prelude::{EncodePacketField, Identifier, LengthPaddedVec};
use macros::{PacketOut, packet_id};

#[derive(Debug, PacketOut)]
#[packet_id(0x01)]
pub struct ClientBoundPluginMessagePacket {
    channel: Identifier,
    data: LengthPaddedVec<i8>,
}

impl ClientBoundPluginMessagePacket {
    pub fn brand(brand: impl ToString) -> Self {
        Self {
            channel: Identifier::minecraft("brand"),
            data: brand
                .to_string()
                .as_bytes()
                .iter()
                .map(|&b| b as i8)
                .collect::<Vec<_>>()
                .into(),
        }
    }
}
