pub trait EncodePacket: Sized {
    fn encode(&self, protocol_version: i32) -> anyhow::Result<Vec<u8>>;
}
