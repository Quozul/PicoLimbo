use pico_identifier::Identifier;

pub struct DimensionInfo {
    pub height: i32,
    pub min_y: i32,
    pub legacy_protocol_id: i8,
    pub protocol_id: u32,
    pub registry_key: Identifier,
}

impl DimensionInfo {
    /// Creates a `DimensionInfo` using the legacy format. All dimensions at this time were 256 blocks tall and started a Y=0.
    #[must_use]
    pub const fn new_legacy(legacy_protocol_id: i8, registry_key: Identifier) -> Self {
        Self {
            height: 256,
            min_y: 0,
            protocol_id: 0,
            legacy_protocol_id,
            registry_key,
        }
    }
}
