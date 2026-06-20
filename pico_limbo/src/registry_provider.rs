use minecraft_protocol::prelude::ProtocolVersion;
use pico_precomputed_registries::PrecomputedRegistries;
use pico_registries::registry_provider::{RegistryProvider, RuntimeRegistryProvider};
use std::path::PathBuf;

#[derive(Clone, Default)]
#[allow(dead_code)]
pub enum RegistryProviderMode {
    RuntimeNbt,
    RuntimeJson,
    #[default]
    BundledBinary = 3,
}

pub const DEFAULT_REGISTRY_PROVIDER_MODE: RegistryProviderMode =
    RegistryProviderMode::BundledBinary;

pub fn load_registry_provider(
    protocol_version: ProtocolVersion,
    mode: &RegistryProviderMode,
) -> pico_registries::Result<Box<dyn RegistryProvider>> {
    let protocol_version = protocol_version.data();
    match mode {
        RegistryProviderMode::RuntimeNbt => {
            let base_path = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap())
                .parent()
                .unwrap()
                .join("data_generator")
                .join("data");
            let provider =
                RuntimeRegistryProvider::new_from_nbt_file(&base_path, protocol_version)?;
            Ok(Box::new(provider) as Box<dyn RegistryProvider>)
        }
        RegistryProviderMode::RuntimeJson => {
            let base_path = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap())
                .parent()
                .unwrap()
                .join("cache")
                .join("generated");
            let provider =
                RuntimeRegistryProvider::new_from_data_generator(&base_path, protocol_version)?;
            Ok(Box::new(provider) as Box<dyn RegistryProvider>)
        }
        RegistryProviderMode::BundledBinary => {
            Ok(Box::new(PrecomputedRegistries::new(protocol_version)))
        }
    }
}
