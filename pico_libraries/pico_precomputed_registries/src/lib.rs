use pico_registries::Identifier;
use pico_registries::registry_provider::DimensionInfo;
use pico_registries::registry_provider::RegistryDataEntry;
use pico_registries::registry_provider::{Dimension, RegistryProvider};
use pico_registries::registry_provider::{RegistryTag, TaggedRegistry};
use pico_registries::{Error, Result};
use protocol_version::protocol_version::ProtocolVersion;

#[allow(clippy::unreadable_literal)]
mod precomputed {
    include!(concat!(env!("OUT_DIR"), "/precomputed_registries.rs"));
}

pub struct PrecomputedRegistries;

impl RegistryProvider for PrecomputedRegistries {
    fn get_biome_protocol_id(
        &self,
        protocol_version: ProtocolVersion,
        biome_identifier: &Identifier,
    ) -> Result<u32> {
        if &biome_identifier.to_string() != "minecraft:plains" {
            return Err(Error::Obf); // Unsupported biome
        }

        let key = format!("{protocol_version:?}");
        precomputed::BIOME_IDS.get(&key).copied().ok_or(Error::Obf) // Version not supported
    }

    fn get_dimension_codec_v1_16_2(
        &self,
        protocol_version: ProtocolVersion,
        dimension: &Dimension,
    ) -> Result<Vec<u8>> {
        let key = format!("{protocol_version:?}");
        let codecs = precomputed::DIMENSION_CODECS.get(&key).ok_or(Error::Obf)?; // Incompatible version

        let slice = match dimension {
            Dimension::Overworld => codecs.overworld,
            Dimension::TheNether => codecs.nether,
            Dimension::TheEnd => codecs.end,
        };

        Ok(slice.to_vec())
    }

    fn get_registry_codec_v1_16(&self, protocol_version: ProtocolVersion) -> Result<Vec<u8>> {
        let key = format!("{protocol_version:?}");
        precomputed::REGISTRY_CODECS
            .get(&key)
            .map(|s| s.to_vec())
            .ok_or(Error::Obf) // Incompatible version
    }

    fn get_dimension_info(
        &self,
        protocol_version: ProtocolVersion,
        dimension_identifier: &Identifier,
    ) -> Result<DimensionInfo> {
        let ver_key = format!("{protocol_version:?}");
        let compound_key = format!("{ver_key}|{dimension_identifier}");

        let info = precomputed::DIMENSION_INFOS
            .get(&compound_key)
            .ok_or(Error::Obf)?; // Unknown dimension or version

        Ok(DimensionInfo {
            height: info.height,
            min_y: info.min_y,
            protocol_id: info.protocol_id,
            registry_key: Identifier::vanilla_unchecked(info.registry_key),
        })
    }

    fn get_registry_data_v1_20_5(
        &self,
        protocol_version: ProtocolVersion,
    ) -> Result<Vec<(Identifier, Vec<RegistryDataEntry>)>> {
        let key = format!("{protocol_version:?}");
        let static_data = precomputed::REGISTRY_DATA.get(&key).ok_or(Error::Obf)?; // Incompatible version

        let result = static_data
            .iter()
            .map(|(id_str, entries)| {
                let ident = Identifier::vanilla_unchecked(*id_str);
                let entries_vec = entries
                    .iter()
                    .map(|e| RegistryDataEntry {
                        entry_id: Identifier::vanilla_unchecked(e.entry_id),
                        nbt_bytes: e.nbt_bytes.to_vec(),
                    })
                    .collect();
                (ident, entries_vec)
            })
            .collect();

        Ok(result)
    }

    fn get_tagged_registries(
        &self,
        protocol_version: ProtocolVersion,
    ) -> Result<Vec<TaggedRegistry>> {
        let key = format!("{protocol_version:?}");
        let static_data = precomputed::TAGGED_REGISTRIES.get(&key).ok_or(Error::Obf)?; // Incompatible version

        let result = static_data
            .iter()
            .map(|reg| TaggedRegistry {
                registry_id: Identifier::vanilla_unchecked(reg.registry_id),
                tags: reg
                    .tags
                    .iter()
                    .map(|t| RegistryTag {
                        identifier: Identifier::vanilla_unchecked(t.identifier),
                        ids: t.ids.to_vec(),
                    })
                    .collect(),
            })
            .collect();

        Ok(result)
    }
}
