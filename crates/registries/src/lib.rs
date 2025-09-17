use minecraft_protocol::prelude::*;
pub use registries_data::grouped_registries::{
    V1_20_5Registries, V1_20_5RegistryEntries, V1_20_5RegistryEntry,
};
pub use registries_data::registry_format::RegistryFormat;

/// This enum holds the pre-encoded, static registries data for a given format.
/// All variants now hold static references, eliminating runtime allocations.
pub enum Registries {
    /// Starting 1.20.5 and onwards, registries are split into several packets
    V1_20_5 { registries: V1_20_5Registries },
    /// Starting 1.20.2 up to 1.20.5 (included) when configuration state was introduced
    /// all registries are sent in the same packet
    V1_20_2 { registry_codec: &'static [u8] },
    /// For versions between 1.19 and 1.20 (included), the dimension is removed, we only send the registry codec
    V1_19 { registry_codec: &'static [u8] },
    /// For versions between 1.16.2 and 1.18.2 (included), we must send the dimension codec separately
    V1_16_2 {
        registry_codec: &'static [u8],
        dimension: &'static [u8],
    },
    /// For 1.16 and 1.16.1, we only have to send the registry codec
    V1_16 { registry_codec: &'static [u8] },
    /// For versions older than 1.16, there are no registries
    None,
}

include!(concat!(env!("OUT_DIR"), "/generated_registries.rs"));

pub fn get_registries(protocol_version: ProtocolVersion, dimension: Dimension) -> Registries {
    let data_version = protocol_version.data();
    get_pregenerated_registries(data_version, dimension)
}

pub fn get_dimension_index(protocol_version: ProtocolVersion, dimension: Dimension) -> Option<i32> {
    let data_version = protocol_version.data();
    if let Some(value) = get_pregenerated_dimension_index(data_version, dimension)
        && let Ok(value) = i32::try_from(value)
    {
        return Some(value);
    }
    None
}

pub fn get_plains_biome_index(protocol_version: ProtocolVersion) -> Option<i32> {
    let data_version = protocol_version.data();
    if let Some(value) = get_pregenerated_plains_biome_index(data_version)
        && let Ok(value) = i32::try_from(value)
    {
        return Some(value);
    }
    None
}
