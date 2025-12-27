use crate::RegistryKeys;
use crate::utils::shared::load_registry_manager;
use pico_identifier::Identifier;
use protocol_version::protocol_version::ProtocolVersion;

///
///
/// # Errors
pub fn get_biome_protocol_id(
    protocol_version: ProtocolVersion,
    biome_identifier: &Identifier,
) -> crate::Result<u32> {
    let registry_manager = load_registry_manager(protocol_version, &[RegistryKeys::DimensionType])?;
    Ok(registry_manager
        .get(&RegistryKeys::Biome)?
        .get(biome_identifier)?
        .get_protocol_id())
}
