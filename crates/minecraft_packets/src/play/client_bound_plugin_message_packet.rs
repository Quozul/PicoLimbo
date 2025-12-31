use minecraft_protocol::prelude::*;

#[derive(PacketOut)]
pub struct PlayClientBoundPluginMessagePacket {
    channel: Identifier,
    data: LengthPaddedVec<i8>,
}

impl PlayClientBoundPluginMessagePacket {
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
