mod data;
mod registry_keys;
mod reports;

pub use crate::data::registry::Registry;
use crate::data::registry_entry::RegistryEntry;
pub use crate::data::registry_manager::{RegistryManager, RegistryManagerBuilder};
pub use crate::data::tag::Tag;
pub use crate::registry_keys::RegistryKeys;
pub use pico_identifier::Identifier;

#[derive(Debug)]
pub struct DimensionInfo {
    pub height: i32,
    pub min_y: i32,
    pub protocol_id: u32,
    pub registry_key: Identifier,
}

pub fn get_dimension_info(
    registry_manager: &RegistryManager,
    dimension_identifier: &Identifier,
) -> Option<DimensionInfo> {
    let registry = registry_manager.get_optional(&RegistryKeys::DimensionType);
    registry
        .and_then(|reg| reg.get_optional(dimension_identifier))
        .and_then(|element| {
            let protocol_id = element.get_protocol_id();
            let registry_key = element.get_registry_key().get_value().clone();
            element.get_dimension().ok().map(|dimension| DimensionInfo {
                height: dimension.get_height(),
                min_y: dimension.get_min_height(),
                protocol_id,
                registry_key,
            })
        })
}

pub fn get_biome_protocol_id(
    registry_manager: &RegistryManager,
    biome_identifier: &Identifier,
) -> Option<u32> {
    let registry = registry_manager.get_optional(&RegistryKeys::Biome);
    registry
        .and_then(|reg| reg.get_optional(biome_identifier))
        .map(RegistryEntry::get_protocol_id)
}
