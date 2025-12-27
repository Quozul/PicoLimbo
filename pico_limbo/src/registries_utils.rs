use minecraft_protocol::prelude::ProtocolVersion;
use pico_registries::{RegistryKeys, RegistryManager};
use std::path::Path;

pub fn load_registry_manager(
    protocol_version: ProtocolVersion,
    registries: &[RegistryKeys],
) -> RegistryManager {
    let path = format!("data/generated/{}", protocol_version.to_string());
    let resource_root = Path::new(&path);
    RegistryManager::builder()
        .register_all(registries)
        .load_from_resource_path(resource_root)
}
