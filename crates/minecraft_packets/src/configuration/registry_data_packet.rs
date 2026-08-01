use crate::configuration::data::registry_entry::RegistryEntry;
use minecraft_protocol::prelude::*;
use std::borrow::Cow;

/// This packet is to use with >= 1.20.2
#[derive(PacketOut)]
pub struct RegistryDataPacket {
    #[protocol_version(min = V1_20_5)]
    registry_id: Omitted<Identifier>,
    #[protocol_version(min = V1_20_5)]
    entries: Omitted<LengthPaddedVec<RegistryEntry>>,
    #[protocol_version(min = V1_20_2, max = V1_20_3)]
    registry_codec_bytes: Omitted<Cow<'static, [u8]>>,
}

impl RegistryDataPacket {
    /// Since 1.20.2 until 1.20.4 (included)
    pub fn codec(registry_codec_bytes: Cow<'static, [u8]>) -> Self {
        Self {
            registry_id: Omitted::None,
            entries: Omitted::None,
            registry_codec_bytes: Omitted::Some(registry_codec_bytes),
        }
    }

    /// Since 1.20.5 (included)
    pub fn registry(registry_id: Identifier, entries: Vec<RegistryEntry>) -> Self {
        Self {
            registry_id: Omitted::Some(registry_id),
            entries: Omitted::Some(LengthPaddedVec::new(entries)),
            registry_codec_bytes: Omitted::None,
        }
    }
}
