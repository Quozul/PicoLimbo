use crate::prelude::{DecodePacket, EncodePacket};
use pico_binutils::prelude::{
    BinaryReader, BinaryReaderError, BinaryWriter, BinaryWriterError, VarIntPrefixedString,
};
use pico_identifier::Identifier;
use protocol_version::protocol_version::ProtocolVersion;

impl DecodePacket for Identifier {
    /// Decodes an identifier.
    /// An identifier is a String with a namespace and a path separated by a colon.
    /// If the namespace is not provided, it defaults to "minecraft".
    fn decode(
        reader: &mut BinaryReader,
        _protocol_version: ProtocolVersion,
    ) -> Result<Self, BinaryReaderError> {
        let decoded_string = reader.read::<VarIntPrefixedString>()?.into_inner();

        let mut split = decoded_string.split(':');
        let namespace = split.next().unwrap_or("minecraft");
        let thing = split.next().unwrap_or("");
        Ok(Self {
            namespace: namespace.to_string(),
            thing: thing.to_string(),
        })
    }
}

impl EncodePacket for Identifier {
    fn encode(
        &self,
        writer: &mut BinaryWriter,
        protocol_version: ProtocolVersion,
    ) -> Result<(), BinaryWriterError> {
        let string = format!("{}:{}", self.namespace, self.thing);
        string.encode(writer, protocol_version)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::{DecodePacket, EncodePacket};

    #[test]
    fn test_identifier() {
        // Given
        let identifier = Identifier::vanilla_unchecked("overworld");
        let mut writer = BinaryWriter::new();
        identifier
            .encode(&mut writer, ProtocolVersion::Any)
            .unwrap();

        let bytes = writer.into_inner();
        let mut reader = BinaryReader::new(&bytes);

        // When
        let decoded_identifier = Identifier::decode(&mut reader, ProtocolVersion::Any).unwrap();

        // Then
        assert_eq!(identifier, decoded_identifier);
    }
}
