use minecraft_protocol::prelude::*;

#[derive(PacketOut)]
pub struct TransferPacket {
    // TODO should this be named PlayTransferPacket since there are also configuration phase transfers?
    pub host: String,
    pub port: VarInt,
}

impl TransferPacket {
    pub fn new(host: &str, port: &VarInt) -> Self {
        Self {
            host: host.to_owned(),
            port: port.clone(),
        }
    }
}
