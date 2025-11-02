use blocks_report::{BlocksReport, ReportIdMapping, build_report_mappings};
use minecraft_protocol::prelude::{BinaryReaderError, ProtocolVersion};
use pico_structures::prelude::{Schematic, SchematicError, World, WorldLoadingError};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use tracing::debug;

#[derive(Debug, Error)]
pub enum LoadWorldError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    BinaryReader(#[from] BinaryReaderError),
    #[error(transparent)]
    Schematic(#[from] SchematicError),
    #[error(transparent)]
    WorldLoading(#[from] WorldLoadingError),
    #[error("Failed to load custom mappings")]
    FailedToLoadCustomMapping, // This can never happen
}

type Result<T> = std::result::Result<T, LoadWorldError>;

fn load_mappings(
    blocks_override: Option<&Path>,
) -> Result<(blocks_report::InternalMapping, Option<ReportIdMapping>)> {
    let (internal, custom_mapping) = match blocks_override {
        None => (blocks_report::load_internal_mapping()?, None),
        Some(override_path) => {
            let report = BlocksReport::from_path(override_path, ProtocolVersion::Any)?;

            let reports = &[report];
            let internal = blocks_report::build_internal_id_mapping(reports);

            let report_mappings = build_report_mappings(reports, &internal);
            let mapping = report_mappings
                .first()
                .ok_or(LoadWorldError::FailedToLoadCustomMapping)?
                .mapping
                .clone();

            (internal, Some(mapping))
        }
    };

    Ok((internal, custom_mapping))
}

fn find_largest_internal_id(internal: &blocks_report::InternalMapping) -> u32 {
    internal.mapping.inner().iter().fold(0u32, |acc, cur| {
        let mut all_ids = cur
            .states
            .inner()
            .iter()
            .map(|state| state.internal_id)
            .collect::<Vec<_>>();
        all_ids.push(cur.default_internal_id);
        let max_id = all_ids.into_iter().max().unwrap_or_default();
        max_id.max(acc)
    })
}

fn load_schematic(
    schematic_path: &Path,
    internal: &blocks_report::InternalMapping,
) -> Result<Schematic> {
    let schem = Schematic::load_schematic_file(schematic_path, internal)?;
    Ok(schem)
}

fn build_world(schematic: &Schematic, biggest_id: u32) -> Result<World> {
    Ok(World::from_schematic(schematic, biggest_id)?)
}

#[derive(Clone, Default)]
pub struct LoadWorldResult {
    pub world: Option<Arc<World>>,
    pub custom_mapping: Option<ReportIdMapping>,
}

pub fn try_load_world(
    schematic_file_path: &str,
    blocks_override: Option<&Path>,
) -> Result<LoadWorldResult> {
    if schematic_file_path.is_empty() {
        return Ok(LoadWorldResult::default());
    }

    let (internal_mapping, custom_mapping) =
        time_operation("Loading block mapping", || load_mappings(blocks_override))?;

    let largest_internal_id = find_largest_internal_id(&internal_mapping);
    debug!("Loaded {largest_internal_id} unique block states");

    let schematic_path = PathBuf::from(schematic_file_path);
    let schematic = time_operation("Loading schematic", || {
        load_schematic(&schematic_path, &internal_mapping)
    })?;

    let world = time_operation("Loading world", || {
        build_world(&schematic, largest_internal_id)
    })?;

    Ok(LoadWorldResult {
        world: Some(Arc::new(world)),
        custom_mapping,
    })
}

fn format_duration(duration: Duration) -> String {
    let total_secs = duration.as_secs_f64();

    if total_secs >= 1.0 {
        format!("{total_secs:.1}s")
    } else {
        format!("{}ms", duration.as_millis())
    }
}

fn time_operation<T, F>(operation_name: &str, operation: F) -> T
where
    F: FnOnce() -> T,
{
    debug!("{operation_name}...");
    let start = std::time::Instant::now();
    let result = operation();
    let elapsed = start.elapsed();
    debug!("Time elapsed: {}", format_duration(elapsed));
    result
}
