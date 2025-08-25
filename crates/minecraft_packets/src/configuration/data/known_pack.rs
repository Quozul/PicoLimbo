use minecraft_protocol::prelude::*;

#[derive(Debug, PacketIn, PacketOut)]
pub struct KnownPack {
    namespace: String,
    id: String,
    version: String,
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
