use crate::RegistryKeys;
use crate::utils::shared::{get_dimension, load_registry_manager};
use pico_identifier::Identifier;
use protocol_version::protocol_version::ProtocolVersion;

pub struct DimensionInfo {
    pub height: i32,
    pub min_y: i32,
    pub protocol_id: u32,
    pub registry_key: Identifier,
}

///
///
/// # Errors
pub fn get_dimension_info(
    protocol_version: ProtocolVersion,
    dimension_identifier: &Identifier,
) -> crate::Result<DimensionInfo> {
    let registry_manager = load_registry_manager(protocol_version, &[RegistryKeys::DimensionType])?;
    let element = get_dimension(&registry_manager, dimension_identifier)?;
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
