use crate::registry_provider::biome::get_biome_protocol_id;
pub use crate::registry_provider::dimension_codec::Dimension;
use crate::registry_provider::dimension_codec::get_dimension_codec_v1_16_2;
pub use crate::registry_provider::dimension_info::DimensionInfo;
use crate::registry_provider::dimension_info::get_dimension_info;
use crate::registry_provider::registry_codec::get_registry_codec_v1_16;
pub use crate::registry_provider::registry_data_v1_20_5::RegistryDataEntry;
use crate::registry_provider::registry_data_v1_20_5::get_registry_data_v1_20_5;
use crate::registry_provider::tagged_registries::get_tagged_registries;
pub use crate::registry_provider::tagged_registries::{RegistryTag, TaggedRegistry};
pub use pico_identifier::Identifier;
use protocol_version::protocol_version::ProtocolVersion;

mod biome;
mod dimension_codec;
mod dimension_info;
mod registry_codec;
mod registry_data_v1_20_5;
mod shared;
mod tagged_registries;

pub trait RegistryProvider {
    ///
    ///
    /// # Errors
    fn get_biome_protocol_id(
        &self,
        protocol_version: ProtocolVersion,
        biome_identifier: &Identifier,
    ) -> crate::Result<u32>;

    /// Dimension codec is a thing from 1.16.2 up to 1.18.2
    ///
    /// # Returns
    /// Serialized NBT of the dimension codec
    ///
    /// # Errors
    fn get_dimension_codec_v1_16_2(
        &self,
        protocol_version: ProtocolVersion,
        dimension: &Dimension,
    ) -> crate::Result<Vec<u8>>;

    /// Since 1.16.0 up until 1.20.4 included, all registries are sent as a single NBT tag
    ///
    /// # Returns
    /// Serialized NBT of the registry codec
    ///
    /// # Errors
    /// Returns an error if this function was called for the wrong protocol version
    fn get_registry_codec_v1_16(&self, protocol_version: ProtocolVersion)
    -> crate::Result<Vec<u8>>;

    ///
    ///
    /// # Errors
    fn get_dimension_info(
        &self,
        protocol_version: ProtocolVersion,
        dimension_identifier: &Identifier,
    ) -> crate::Result<DimensionInfo>;

    /// Since 1.20.5, each registry is sent in its own packet
    ///
    /// # Errors
    fn get_registry_data_v1_20_5(
        &self,
        protocol_version: ProtocolVersion,
    ) -> crate::Result<Vec<(Identifier, Vec<RegistryDataEntry>)>>;

    ///
    ///
    /// # Errors
    fn get_tagged_registries(
        &self,
        protocol_version: ProtocolVersion,
    ) -> crate::Result<Vec<TaggedRegistry>>;
}

pub struct RuntimeRegistryProvider;

impl RegistryProvider for RuntimeRegistryProvider {
    fn get_biome_protocol_id(
        &self,
        protocol_version: ProtocolVersion,
        biome_identifier: &Identifier,
    ) -> crate::Result<u32> {
        get_biome_protocol_id(protocol_version, biome_identifier)
    }

    fn get_dimension_codec_v1_16_2(
        &self,
        protocol_version: ProtocolVersion,
        dimension: &Dimension,
    ) -> crate::Result<Vec<u8>> {
        get_dimension_codec_v1_16_2(protocol_version, dimension)
    }

    fn get_registry_codec_v1_16(
        &self,
        protocol_version: ProtocolVersion,
    ) -> crate::Result<Vec<u8>> {
        get_registry_codec_v1_16(protocol_version)
    }

    fn get_dimension_info(
        &self,
        protocol_version: ProtocolVersion,
        dimension_identifier: &Identifier,
    ) -> crate::Result<DimensionInfo> {
        get_dimension_info(protocol_version, dimension_identifier)
    }

    fn get_registry_data_v1_20_5(
        &self,
        protocol_version: ProtocolVersion,
    ) -> crate::Result<Vec<(Identifier, Vec<RegistryDataEntry>)>> {
        get_registry_data_v1_20_5(protocol_version)
    }

    fn get_tagged_registries(
        &self,
        protocol_version: ProtocolVersion,
    ) -> crate::Result<Vec<TaggedRegistry>> {
        get_tagged_registries(protocol_version)
    }
}
