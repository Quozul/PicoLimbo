use minecraft_protocol::prelude::{
    BinaryWriter, BinaryWriterError, EncodePacket, Identifiable, ProtocolVersion,
};
use std::fmt::Display;
use thiserror::Error;

#[derive(Debug, Default)]
pub struct RawPacket {
    data: Vec<u8>,
}

#[derive(Error, Debug)]
pub enum RawPacketError {
    #[error("invalid packet length")]
    InvalidPacketLength,
    #[error("failed to encode packet {id} for version {version}")]
    EncodePacket { id: u8, version: i32 },
}

impl RawPacket {
    /// Creates a raw packet, containing its ID and associated data.
    /// The data vector must not be length padded.
    pub fn new(data: Vec<u8>) -> Result<Self, RawPacketError> {
        if data.is_empty() {
            Err(RawPacketError::InvalidPacketLength)
        } else {
            Ok(Self { data })
        }
    }

    pub fn from_bytes(packet_id: u8, bytes: &[u8]) -> Self {
        let mut data = vec![packet_id];
        data.append(&mut bytes.to_vec());
        Self { data }
    }

    /// Creates a new raw packet from a serializable packet struct.
    pub fn from_packet<T>(
        packet_id: u8,
        version_number: i32,
        packet: &T,
    ) -> Result<Self, BinaryWriterError>
    where
        T: EncodePacket + Identifiable,
    {
        let mut writer = BinaryWriter::new();
        writer.write(&packet_id)?;
        packet.encode(&mut writer, ProtocolVersion::from(version_number))?;

        let data = writer.into_inner();
        Ok(Self { data })
    }

    pub const fn size(&self) -> usize {
        self.data.len()
    }

    pub fn packet_id(&self) -> Option<u8> {
        self.data.first().copied()
    }

    pub fn data(&self) -> &[u8] {
        if self.data.is_empty() {
            &[]
        } else {
            &self.data[1..]
        }
    }

    pub fn bytes(&self) -> &[u8] {
        &self.data
    }
}

impl Display for RawPacket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for byte in self.data() {
            write!(f, "{byte:02X} ")?;
        }
        Ok(())
    }
}
