mod registries_report;
mod registry_data;
mod registry_keys;

use std::path::Path;

use crate::registries_report::RegistriesReport;
use crate::registry_data::RegistryManager;
use crate::registry_keys::RegistryKeys;
use minecraft_protocol::prelude::Identifier;
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
    let registry_manager =
        RegistryManager::from_generated_directory(Path::new("data/generated/V1_21_6/data"));
    let registry = registry_manager.get_optional(RegistryKeys::DimensionType);
    let dimension = registry
        .and_then(|reg| reg.get_optional(Identifier::minecraft("overworld")))
        .and_then(|element| element.get_dimension().ok());
    debug!("dimension: {:?}", dimension);
}

fn debug_registries_report() {
    let registries =
        RegistriesReport::from_generated_directory(Path::new("/data/generated/V1_21_6"));
    debug!(?registries);
}
