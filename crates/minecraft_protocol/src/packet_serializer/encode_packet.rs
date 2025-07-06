pub trait EncodePacket: Sized {
    fn encode(&self, protocol_version: u32) -> anyhow::Result<Vec<u8>>;
}
