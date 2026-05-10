use minecraft_protocol::prelude::*;

#[derive(PacketIn, PacketOut)]
pub struct KnownPack {
    pub namespace: String,
    pub id: String,
    pub version: String,
}

impl KnownPack {
    pub fn new(version: &str) -> Self {
        Self {
            namespace: "minecraft".to_string(),
            id: "core".to_string(),
            version: version.to_string(),
        }
    }
}
