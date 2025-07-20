use minecraft_protocol::prelude::*;

#[derive(Debug, PacketOut)]
#[packet_id("configuration/clientbound/minecraft:custom_payload")]
pub struct ConfigurationClientBoundPluginMessagePacket {
    channel: Identifier,
    data: LengthPaddedVec<i8>,
}

impl ConfigurationClientBoundPluginMessagePacket {
    pub fn brand(brand: impl ToString) -> Self {
        Self {
            channel: Identifier::minecraft("brand"),
            data: LengthPaddedVec::new(
                brand
                    .to_string()
                    .as_bytes()
                    .iter()
                    .map(|&b| b as i8)
                    .collect::<Vec<_>>(),
            ),
        }
    }
}
