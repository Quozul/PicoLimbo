use minecraft_protocol::prelude::*;

#[derive(Debug, PacketIn, PacketOut)]
pub struct KnownPack {
    pub namespace: String,
    pub id: String,
    pub version: String,
}

impl Default for KnownPack {
    fn default() -> Self {
        Self {
            namespace: "minecraft".to_string(),
            id: "core".to_string(),
            version: "1.21.4".to_string(),
        }
    }
}
