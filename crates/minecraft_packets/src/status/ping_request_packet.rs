use minecraft_protocol::prelude::*;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, PacketIn)]
#[packet_id("status/serverbound/minecraft:ping_request")]
pub struct PingRequestPacket {
    pub timestamp: i64,
}

impl Default for PingRequestPacket {
    fn default() -> Self {
        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        Self {
            timestamp: since_the_epoch.as_secs() as i64,
        }
    }
}
