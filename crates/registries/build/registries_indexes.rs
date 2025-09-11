use crate::get_all_registries::get_version_directory;
use minecraft_protocol::prelude::ProtocolVersion;
use std::path::Path;

fn get_entry_index(
    protocol_version: ProtocolVersion,
    data_location: &Path,
    subdirectory: &str,
    entry_name: &str,
    default: usize,
) -> usize {
    let directory = get_version_directory(protocol_version, data_location).join(subdirectory);

    if !directory.is_dir() {
        return 1;
    }

    directory.read_dir().map_or(default, |entries| {
        entries
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let path = entry.path();
                if path.is_file() {
                    path.file_stem()
                        .and_then(|stem| stem.to_str())
                        .map(|s| s.to_string())
                } else {
                    None
                }
            })
            .position(|entry| entry == entry_name)
            .unwrap_or(default)
    })
}

// Only for V1_20_3 and above
pub fn get_the_void_index(protocol_version: ProtocolVersion, data_location: &Path) -> usize {
    get_entry_index(
        protocol_version,
        data_location,
        "worldgen/biome",
        "plains",
        1,
    )
}

// Only for V1_20_5 and above
pub fn get_dimension_type_index(
    protocol_version: ProtocolVersion,
    data_location: &Path,
    dimension_name: String,
) -> usize {
    get_entry_index(
        protocol_version,
        data_location,
        "dimension_type",
        &dimension_name,
        0,
    )
}
