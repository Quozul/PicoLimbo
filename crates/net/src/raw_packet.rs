use minecraft_protocol::prelude::{EncodePacket, PacketId};
use std::fmt::Display;
use thiserror::Error;

#[derive(Debug)]
pub struct RawPacket {
    data: Vec<u8>,
}

#[derive(Error, Debug)]
pub enum RawPacketError {
    #[error("invalid packet length")]
    InvalidPacketLength,
    #[error("failed to encode packet {id} for version {version}")]
    EncodePacket { id: u8, version: u32 },
}

impl RawPacket {
    /// Creates a raw packet, containing its ID and associated data.
    /// The data vector must not be length padded.
    pub fn new(data: Vec<u8>) -> Result<Self, RawPacketError> {
        if data.is_empty() {
            Err(RawPacketError::InvalidPacketLength)
        } else {
            Ok(RawPacket { data })
        }
    }

    /// Creates a new raw packet from a serializable packet struct.
    pub fn from_packet<T>(
        packet_id: u8,
        version_number: u32,
        packet: &T,
    ) -> Result<Self, RawPacketError>
    where
        T: EncodePacket + PacketId,
    {
        let mut data = vec![packet_id];
        let encoded_packet =
            packet
                .encode(version_number)
                .map_err(|_| RawPacketError::EncodePacket {
                    id: packet_id,
                    version: version_number,
                })?;
        data.extend_from_slice(&encoded_packet);
        Ok(Self { data })
    }

    pub fn size(&self) -> usize {
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
}

impl Display for RawPacket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for byte in self.data() {
            write!(f, "{byte:02X} ")?;
        }
        Ok(())
    }
}
