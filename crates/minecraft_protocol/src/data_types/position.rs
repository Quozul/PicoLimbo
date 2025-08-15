use crate::prelude::{DecodePacket, EncodePacket};
use pico_binutils::prelude::{BinaryReader, BinaryReaderError, BinaryWriter, BinaryWriterError};
use protocol_version::protocol_version::ProtocolVersion;

#[derive(Debug, Default)]
pub struct Position {
    x: f64,
    y: f64,
    z: f64,
}

impl Position {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Position { x, y, z }
    }
}

impl DecodePacket for Position {
    fn decode(
        reader: &mut BinaryReader,
        protocol_version: ProtocolVersion,
    ) -> Result<Self, BinaryReaderError> {
        let val = i64::decode(reader, protocol_version)?;
        let x = (val >> 38) as f64;
        let y = (val << 52 >> 52) as f64;
        let z = (val << 26 >> 38) as f64;
        Ok(Position { x, y, z })
    }
}

impl EncodePacket for Position {
    fn encode(
        &self,
        writer: &mut BinaryWriter,
        _protocol_version: ProtocolVersion,
    ) -> Result<(), BinaryWriterError> {
        let value = ((self.x as i64 & 0x3FFFFFF) << 38)
            | ((self.z as i64 & 0x3FFFFFF) << 12)
            | (self.y as i64 & 0xFFF);
        writer.write(&value)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_position() {
        let position = Position::new(18357644.0, 831.0, -20882616.0);
        let mut writer = BinaryWriter::new();
        position.encode(&mut writer, ProtocolVersion::Any).unwrap();

        let bytes = writer.into_inner();
        let mut reader = BinaryReader::new(&bytes);

        let decoded_position = Position::decode(&mut reader, ProtocolVersion::Any).unwrap();

        assert_eq!(position.x, decoded_position.x);
        assert_eq!(position.y, decoded_position.y);
        assert_eq!(position.z, decoded_position.z);
    }
}
