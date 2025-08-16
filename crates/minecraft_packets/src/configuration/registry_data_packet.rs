use minecraft_protocol::data::registry::registry_entry::RegistryEntry;
use minecraft_protocol::prelude::*;

/// This packet is to use with >= 1.20.2
#[derive(PacketOut)]
pub struct RegistryDataPacket {
    #[pvn(766..)]
    registry_id: Omitted<Identifier>,
    #[pvn(766..)]
    entries: Omitted<LengthPaddedVec<RegistryEntry>>,
    #[pvn(764..766)]
    registry_codec: Omitted<Nbt>,
}

impl RegistryDataPacket {
    pub fn codec(registry_codec: Nbt) -> Self {
        Self {
            registry_id: Omitted::None,
            entries: Omitted::None,
            registry_codec: Omitted::Some(registry_codec),
        }
    }

    pub fn registry(registry_id: Identifier, entries: Vec<RegistryEntry>) -> Self {
        Self {
            registry_id: Omitted::Some(registry_id),
            entries: Omitted::Some(LengthPaddedVec::new(entries)),
            registry_codec: Omitted::None,
        }
    }
}
