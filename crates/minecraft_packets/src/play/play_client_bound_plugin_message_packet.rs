use minecraft_protocol::prelude::*;

#[derive(Debug, PacketOut)]
#[packet_id("play/clientbound/minecraft:custom_payload")]
pub struct PlayClientBoundPluginMessagePacket {
    channel: Identifier,
    data: LengthPaddedVec<i8>,
}

impl PlayClientBoundPluginMessagePacket {
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
