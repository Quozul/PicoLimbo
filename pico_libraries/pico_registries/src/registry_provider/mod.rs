pub use crate::registry_provider::dimension::Dimension;
pub use crate::registry_provider::dimension_info::DimensionInfo;
pub use crate::registry_provider::registry_data_v1_20_5::RegistryDataEntry;
pub use crate::registry_provider::tagged_registries::{RegistryTag, TaggedRegistry};
pub use pico_identifier::Identifier;
pub use runtime_registry_provider::RuntimeRegistryProvider;
use std::borrow::Cow;

mod dimension;
mod dimension_info;
mod registry_codec;
mod registry_data_v1_20_5;
mod runtime_registry_provider;
mod shared;
mod tagged_registries;

pub trait RegistryProvider {
    ///
    ///
    /// # Errors
    fn get_biome_protocol_id(&self, biome_identifier: &Identifier) -> crate::Result<u32>;

    /// Dimension codec is a thing from 1.16.2 up to 1.18.2
    ///
    /// # Returns
    /// Serialized NBT of the dimension codec
    ///
    /// # Errors
    fn get_dimension_codec_v1_16_2(
        &self,
        dimension: Dimension,
    ) -> crate::Result<Cow<'static, [u8]>>;

    /// Since 1.16.0 up until 1.20.4 included, all registries are sent as a single NBT tag
    ///
    /// # Returns
    /// Serialized NBT of the registry codec
    ///
    /// # Errors
    /// Returns an error if this function was called for the wrong protocol version
    fn get_registry_codec_v1_16(&self) -> crate::Result<Cow<'static, [u8]>>;

    ///
    ///
    /// # Errors
    fn get_dimension_info(&self, dimension_identifier: Dimension) -> crate::Result<DimensionInfo>;

    /// Since 1.20.5, each registry is sent in its own packet
    ///
    /// # Errors
    fn get_registry_data_v1_20_5(&self)
    -> crate::Result<Vec<(Identifier, Vec<RegistryDataEntry>)>>;

    ///
    ///
    /// # Errors
    fn get_tagged_registries(&self) -> crate::Result<Vec<TaggedRegistry>>;
}
