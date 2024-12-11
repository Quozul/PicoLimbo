pub trait EncodePacket: Sized {
    fn encode(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>>;
}
