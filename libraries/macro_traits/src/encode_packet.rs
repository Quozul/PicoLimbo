pub trait EncodePacket: Sized {
    fn encode(
        &self,
        protocol_version: &protocol_version::ProtocolVersion,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>>;
}
