use crate::data::dimension_type::DimensionType;
use serde::Serialize;

/// Values of a `RegistryEntry`
/// Only values we care about are handled, you may be interested in using `RegistryEntry::raw_value` for other types
#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum RegistryEntryValue {
    DimensionType(DimensionType),
    Other,
}
