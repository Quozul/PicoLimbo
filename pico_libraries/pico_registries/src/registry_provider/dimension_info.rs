use pico_identifier::Identifier;

pub struct DimensionInfo {
    pub height: i32,
    pub min_y: i32,
    pub protocol_id: u32,
    pub registry_key: Identifier,
}
