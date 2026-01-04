use pico_registries::Identifier;
use pico_registries::registry_provider::RegistryDataEntry;
use pico_registries::registry_provider::RegistryProvider;
use pico_registries::registry_provider::{Dimension, DimensionInfo};
use pico_registries::registry_provider::{RegistryTag, TaggedRegistry};
use pico_registries::{Error, Result};
use protocol_version::protocol_version::ProtocolVersion;
use std::borrow::Cow;

#[allow(clippy::unreadable_literal)]
mod precomputed {
    include!(concat!(env!("OUT_DIR"), "/precomputed_registries.rs"));
}

pub struct PrecomputedRegistries {
    protocol_version: ProtocolVersion,
}

impl PrecomputedRegistries {
    #[must_use]
    pub const fn new(protocol_version: ProtocolVersion) -> Self {
        Self { protocol_version }
    }
}

impl RegistryProvider for PrecomputedRegistries {
    fn get_biome_protocol_id(&self, biome_identifier: &Identifier) -> Result<u32> {
        if &biome_identifier.to_string() != "minecraft:plains" {
            return Err(Error::UnsupportedBiome);
        }

        let key = format!("{:?}", self.protocol_version);
        precomputed::BIOME_IDS
            .get(&key)
            .copied()
            .ok_or(Error::BiomeIdUnsupportedVersion)
    }

    fn get_dimension_codec_v1_16_2(&self, dimension: Dimension) -> Result<Cow<'static, [u8]>> {
        let key = format!("{:?}", self.protocol_version);
        let codecs = precomputed::DIMENSION_CODECS
            .get(&key)
            .ok_or(Error::DimensionCodecUnsupportedVersion)?;

        let slice = match dimension {
            Dimension::Overworld => codecs.overworld,
            Dimension::Nether => codecs.nether,
            Dimension::End => codecs.end,
        };

        Ok(Cow::Borrowed(slice))
    }

    fn get_registry_codec_v1_16(&self) -> Result<Cow<'static, [u8]>> {
        let key = format!("{:?}", self.protocol_version);
        precomputed::REGISTRY_CODECS
            .get(&key)
            .map(|s| Cow::Borrowed(*s))
            .ok_or(Error::RegistryCodecUnsupportedVersion)
    }

    fn get_dimension_info(&self, dimension_identifier: Dimension) -> Result<DimensionInfo> {
        let key = format!("{:?}", self.protocol_version);
        let codecs = precomputed::DIMENSION_INFOS
            .get(&key)
            .ok_or(Error::DimensionInfoUnsupportedVersion)?;

        let info = match dimension_identifier {
            Dimension::Overworld => &codecs.overworld,
            Dimension::Nether => &codecs.nether,
            Dimension::End => &codecs.end,
        };

        Ok(DimensionInfo {
            height: info.height,
            min_y: info.min_y,
            protocol_id: info.protocol_id,
            registry_key: Identifier::vanilla_unchecked(info.registry_key),
        })
    }

    fn get_registry_data_v1_20_5(&self) -> Result<Vec<(Identifier, Vec<RegistryDataEntry>)>> {
        let key = format!("{:?}", self.protocol_version);
        let static_data = precomputed::REGISTRY_DATA
            .get(&key)
            .ok_or(Error::RegistryDataUnsupportedVersion)?;

        let result = static_data
            .iter()
            .map(|(id_str, entries)| {
                let ident = Identifier::vanilla_unchecked(*id_str);
                let entries_vec = entries
                    .iter()
                    .map(|e| RegistryDataEntry {
                        entry_id: Identifier::vanilla_unchecked(e.entry_id),
                        nbt_bytes: Cow::Borrowed(e.nbt_bytes),
                    })
                    .collect();
                (ident, entries_vec)
            })
            .collect();

        Ok(result)
    }

    fn get_tagged_registries(&self) -> Result<Vec<TaggedRegistry>> {
        let key = format!("{:?}", self.protocol_version);
        let static_data = precomputed::TAGGED_REGISTRIES
            .get(&key)
            .ok_or(Error::TaggedRegistriesUnsupportedVersion)?;

        let result = static_data
            .iter()
            .map(|reg| TaggedRegistry {
                registry_id: Identifier::vanilla_unchecked(reg.registry_id),
                tags: reg
                    .tags
                    .iter()
                    .map(|t| RegistryTag {
                        identifier: Identifier::vanilla_unchecked(t.identifier),
                        ids: Cow::Borrowed(t.ids),
                    })
                    .collect(),
            })
            .collect();

        Ok(result)
    }
}
