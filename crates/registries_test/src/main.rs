mod registries_report;
mod registry_data;
mod registry_keys;

use crate::registries_report::RegistriesReport;
use crate::registry_data::RegistryManager;
use crate::registry_keys::RegistryKeys;
use minecraft_protocol::prelude::Identifier;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;
use tracing::{Level, debug};
use tracing_subscriber::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use walkdir::WalkDir;

fn main() {
    let log_level = Level::TRACE;
    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env().add_directive(log_level.into()))
        .with(tracing_subscriber::fmt::layer().with_target(false))
        .init();
    let resource_root = Path::new("data/generated/V1_21_11-pre2/data");
    let registry_manager = RegistryManager::from_generated_directory(resource_root);
    // let dimension = get_dimension_info(&registry_manager, Identifier::minecraft("overworld"));
    // debug!("dimension: {:?}", dimension);
    let tag_groups = tag_group_loader(resource_root);
    debug!(?tag_groups);
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
    dimension_name: Identifier,
) -> Option<DimensionType> {
    let registry = registry_manager.get_optional(RegistryKeys::DimensionType);
    if let Some(element) = registry.and_then(|reg| reg.get_optional(dimension_name)) {
        let protocol_id = element.get_protocol_id();
        let registry_key = element.get_registry_key().get_value().clone();
        element.get_dimension().ok().map(|dimension| DimensionType {
            height: dimension.get_height(),
            min_y: dimension.get_min_height(),
            protocol_id,
            registry_key,
        })
    } else {
        None
    }
}

#[derive(Debug)]
struct TagGroup {
    tag_registry: RegistryKeys,
    // TODO: Avoid usage of HashMap
    tags: HashMap<Identifier, Tag>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Tag {
    values: Vec<String>,
}

fn tag_group_loader(resource_root_path: &Path) -> Vec<TagGroup> {
    const REGISTRY_KEYS: &[RegistryKeys] = &[RegistryKeys::BannerPattern];
    let x = REGISTRY_KEYS
        .iter()
        .map(|registry_keys| {
            let tag_group_path = resource_root_path
                .join(registry_keys.id().namespace)
                .join(registry_keys.get_tag_path());
            let tags = WalkDir::new(&tag_group_path)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| {
                    e.file_type().is_file()
                        && e.path().extension().and_then(|e| e.to_str()) == Some("json")
                })
                .map(|dir_entry| {
                    let path = dir_entry.path();
                    let json_str =
                        std::fs::read_to_string(path).expect("failed to read registries.json");
                    let tag = serde_json::from_str::<Tag>(&json_str)
                        .expect("failed to deserialize dimension type");
                    // TODO: Find a cleaner way to make this conversion from path to identifier
                    // TODO: Handle \ on Windows
                    let file_no_ext = path
                        .strip_prefix(&tag_group_path)
                        .expect("failed to strip prefix")
                        .with_extension("");
                    let file_stem = file_no_ext.to_str().expect("failed to convert file stem");
                    let tag_identifier = Identifier::new(&registry_keys.id().namespace, file_stem);
                    (tag_identifier, tag)
                })
                .collect::<HashMap<Identifier, Tag>>();
            TagGroup {
                tag_registry: *registry_keys,
                tags,
            }
        })
        .collect::<Vec<_>>();
    x
}

fn debug_registries_report() {
    let registries =
        RegistriesReport::from_generated_directory(Path::new("data/generated/V1_21_11-pre2"));
    debug!(?registries);
}
