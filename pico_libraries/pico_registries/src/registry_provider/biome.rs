use crate::RegistryKeys;
use crate::registry_provider::shared::load_registry_manager;
use pico_identifier::Identifier;
use protocol_version::protocol_version::ProtocolVersion;

pub fn get_biome_protocol_id(
    protocol_version: ProtocolVersion,
    biome_identifier: &Identifier,
) -> crate::Result<u32> {
    let registry_manager = load_registry_manager(protocol_version, &[RegistryKeys::Biome])?;
    Ok(registry_manager
        .get(&RegistryKeys::Biome)?
        .get(biome_identifier)?
        .get_protocol_id())
}
