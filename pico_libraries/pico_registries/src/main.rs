#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
mod data;
mod registry_keys;
mod reports;

use crate::data::registry_entry::RegistryEntry;
use crate::data::registry_manager::RegistryManager;
use crate::registry_keys::RegistryKeys;
use pico_identifier::prelude::Identifier;
use std::path::Path;
use tracing::{Level, debug};
use tracing_subscriber::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

fn main() {
    let log_level = Level::TRACE;
    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env().add_directive(log_level.into()))
        .with(tracing_subscriber::fmt::layer().with_target(false))
        .init();

    let resource_root = Path::new("data/generated/V1_21_11");
    let registry_manager = RegistryManager::builder()
        .register(RegistryKeys::Biome)
        .register(RegistryKeys::DimensionType)
        .load_from_resource_path(resource_root);

    let dimension = get_dimension_info(
        &registry_manager,
        &Identifier::vanilla_unchecked("overworld"),
    );
    debug!(?dimension);

    let plains_id =
        get_biome_protocol_id(&registry_manager, &Identifier::vanilla_unchecked("plains"));
    debug!(?plains_id);

    // let registries = RegistriesReport::from_resource_path(resource_root);
    // debug!("{registries:#?}");
}

#[derive(Debug)]
pub struct DimensionType {
    height: i32,
    min_y: i32,
    protocol_id: u32,
    registry_key: Identifier,
}

fn get_dimension_info(
    registry_manager: &RegistryManager,
    dimension_identifier: &Identifier,
) -> Option<DimensionType> {
    let registry = registry_manager.get_optional(RegistryKeys::DimensionType);
    registry
        .and_then(|reg| reg.get_optional(dimension_identifier))
        .and_then(|element| {
            let protocol_id = element.get_protocol_id();
            let registry_key = element.get_registry_key().get_value().clone();
            element.get_dimension().ok().map(|dimension| DimensionType {
                height: dimension.get_height(),
                min_y: dimension.get_min_height(),
                protocol_id,
                registry_key,
            })
        })
}

fn get_biome_protocol_id(
    registry_manager: &RegistryManager,
    biome_identifier: &Identifier,
) -> Option<u32> {
    let registry = registry_manager.get_optional(RegistryKeys::Biome);
    registry
        .and_then(|reg| reg.get_optional(biome_identifier))
        .map(RegistryEntry::get_protocol_id)
}
