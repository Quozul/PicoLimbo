use crate::registry_provider::registry_codec::get_registry_codec_v1_16;
use crate::registry_provider::registry_data_v1_20_5::get_registry_data_v1_20_5;
use crate::registry_provider::shared::{
    encode_nameless_compound_to_bytes, get_dimension, get_registry_keys, load_registry_manager,
};
use crate::registry_provider::tagged_registries::get_tagged_registries;
use crate::registry_provider::{
    Dimension, DimensionInfo, RegistryDataEntry, RegistryProvider, TaggedRegistry,
};
use crate::{RegistryKeys, RegistryManager};
use pico_identifier::Identifier;
use protocol_version::protocol_version::ProtocolVersion;
use std::borrow::Cow;
use std::path::Path;

pub struct RuntimeRegistryProvider {
    registry_manager: RegistryManager,
    protocol_version: ProtocolVersion,
}

impl RuntimeRegistryProvider {
    /// Initialize a new registry provider that reads the file system at runtime
    ///
    /// # Errors
    pub fn new(base_path: &Path, protocol_version: ProtocolVersion) -> crate::Result<Self> {
        let registry_keys = get_registry_keys(protocol_version)?;
        Ok(Self {
            registry_manager: load_registry_manager(base_path, protocol_version, &registry_keys)?,
            protocol_version,
        })
    }
}

impl RegistryProvider for RuntimeRegistryProvider {
    fn get_biome_protocol_id(&self, biome_identifier: &Identifier) -> crate::Result<u32> {
        Ok(self
            .registry_manager
            .get(&RegistryKeys::Biome)?
            .get(biome_identifier)?
            .get_protocol_id())
    }

    fn get_dimension_codec_v1_16_2(
        &self,
        dimension: Dimension,
    ) -> crate::Result<Cow<'static, [u8]>> {
        crate::Error::incompatible_version(
            self.protocol_version,
            ProtocolVersion::V1_16_2,
            ProtocolVersion::V1_18_2,
        )?;
        let entry = get_dimension(&self.registry_manager, dimension)?;
        Ok(encode_nameless_compound_to_bytes(
            self.protocol_version,
            &entry.get_raw_value(),
        )?)
    }

    fn get_registry_codec_v1_16(&self) -> crate::Result<Cow<'static, [u8]>> {
        get_registry_codec_v1_16(&self.registry_manager, self.protocol_version)
    }

    fn get_dimension_info(&self, dimension_identifier: Dimension) -> crate::Result<DimensionInfo> {
        let element = get_dimension(&self.registry_manager, dimension_identifier)?;
        let dimension = element.get_dimension()?;
        let protocol_id = element.get_protocol_id();
        let registry_key = element.get_registry_key().get_value().clone();
        Ok(DimensionInfo {
            height: dimension.get_height(),
            min_y: dimension.get_min_height(),
            protocol_id,
            registry_key,
        })
    }

    fn get_registry_data_v1_20_5(
        &self,
    ) -> crate::Result<Vec<(Identifier, Vec<RegistryDataEntry>)>> {
        get_registry_data_v1_20_5(&self.registry_manager, self.protocol_version)
    }

    fn get_tagged_registries(&self) -> crate::Result<Vec<TaggedRegistry>> {
        Ok(get_tagged_registries(&self.registry_manager))
    }
}
