use minecraft_protocol::prelude::{
    BinaryWriter, BinaryWriterError, EncodePacket, ProtocolVersion, VarInt,
};

pub struct EncodeAsBytes<T>(T);

impl<T> EncodeAsBytes<T> {
    pub fn new(data: T) -> Self {
        Self(data)
    }
}

impl<T> EncodePacket for EncodeAsBytes<T>
where
    T: EncodePacket,
{
    fn encode(
        &self,
        writer: &mut BinaryWriter,
        protocol_version: ProtocolVersion,
    ) -> Result<(), BinaryWriterError> {
        let mut payload_writer = BinaryWriter::default();
        self.0.encode(&mut payload_writer, protocol_version)?;

        let payload_size = VarInt::new(payload_writer.len() as i32);
        payload_size.encode(writer, protocol_version)?;

        writer.write_bytes(payload_writer.as_slice())?;
        Ok(())
    }
}
