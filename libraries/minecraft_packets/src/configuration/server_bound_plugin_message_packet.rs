use data_types::prelude::{DecodePacketField, Identifier, LengthPaddedVec};
use macros::{PacketIn, packet_id};

#[derive(Debug, PacketIn)]
#[packet_id(0x02)]
pub struct ServerBoundPluginMessagePacket {
    channel: Identifier,
    data: LengthPaddedVec<i8>,
}
