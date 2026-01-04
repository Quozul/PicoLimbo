use crate::configuration::data::registry_entry::RegistryEntry;
use minecraft_protocol::prelude::*;
use std::borrow::Cow;

/// This packet is to use with >= 1.20.2
#[derive(PacketOut)]
pub struct RegistryDataPacket {
    #[pvn(766..)]
    registry_id: Omitted<Identifier>,
    #[pvn(766..)]
    entries: Omitted<LengthPaddedVec<RegistryEntry>>,
    #[pvn(764..766)]
    registry_codec_bytes: Omitted<Cow<'static, [u8]>>,
}

impl RegistryDataPacket {
    pub fn codec(registry_codec_bytes: Cow<'static, [u8]>) -> Self {
        Self {
            registry_id: Omitted::None,
            entries: Omitted::None,
            registry_codec_bytes: Omitted::Some(registry_codec_bytes),
        }
    }

    pub fn registry(registry_id: Identifier, entries: Vec<RegistryEntry>) -> Self {
        Self {
            registry_id: Omitted::Some(registry_id),
            entries: Omitted::Some(LengthPaddedVec::new(entries)),
            registry_codec_bytes: Omitted::None,
        }
    }
}
