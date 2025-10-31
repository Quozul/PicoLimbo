use minecraft_protocol::prelude::*;

#[derive(PacketOut)]
pub struct SetEntityMetadataPacket {
    entity_id: VarInt,
    entity_metadata: Vec<EntityMetadata>,
}

impl SetEntityMetadataPacket {
    pub fn skin_layers(entity_id: i32) -> Self {
        let entity_metadata = vec![
            EntityMetadata::SkinParts(Metadata::Byte(
                0x01 | 0x02 | 0x04 | 0x08 | 0x10 | 0x20 | 0x40,
            )),
            EntityMetadata::End,
        ];

        Self {
            entity_id: entity_id.into(),
            entity_metadata,
        }
    }
}

enum EntityMetadata {
    SkinParts(Metadata),
    End,
}

impl EntityMetadata {
    fn get_index(&self, protocol_version: ProtocolVersion) -> u8 {
        match self {
            Self::SkinParts(_) => {
                if protocol_version.is_after_inclusive(ProtocolVersion::V1_21_9) {
                    16
                } else if protocol_version.is_after_inclusive(ProtocolVersion::V1_17) {
                    17
                } else if protocol_version.is_after_inclusive(ProtocolVersion::V1_15) {
                    16
                } else if protocol_version.is_after_inclusive(ProtocolVersion::V1_14) {
                    15
                } else if protocol_version.is_after_inclusive(ProtocolVersion::V1_12) {
                    13
                } else if protocol_version.is_after_inclusive(ProtocolVersion::V1_9) {
                    15
                } else if protocol_version.is_after_inclusive(ProtocolVersion::V1_8) {
                    10
                } else {
                    panic!("Unsupported protocol version");
                }
            }
            Self::End => {
                if protocol_version.is_after_inclusive(ProtocolVersion::V1_9) {
                    255
                } else {
                    127
                }
            }
        }
    }
}

impl EncodePacket for EntityMetadata {
    fn encode(
        &self,
        writer: &mut BinaryWriter,
        protocol_version: ProtocolVersion,
    ) -> Result<(), BinaryWriterError> {
        self.get_index(protocol_version)
            .encode(writer, protocol_version)?;
        match self {
            EntityMetadata::SkinParts(metadata) => {
                metadata.encode(writer, protocol_version)?;
            }
            EntityMetadata::End => {}
        }
        Ok(())
    }
}

#[derive(Clone)]
enum Metadata {
    Byte(i8),
}

impl Metadata {
    fn get_type_id(&self) -> u8 {
        match self {
            Metadata::Byte(_) => 0,
        }
    }
}

impl EncodePacket for Metadata {
    fn encode(
        &self,
        writer: &mut BinaryWriter,
        protocol_version: ProtocolVersion,
    ) -> Result<(), BinaryWriterError> {
        let type_id = self.get_type_id();
        let final_byte = if protocol_version.is_after_inclusive(ProtocolVersion::V1_9) {
            type_id.encode(writer, protocol_version)?;
            match self {
                Self::Byte(value) => *value as u8,
            }
        } else {
            match self {
                Self::Byte(value) => (type_id << 5) | (*value as u8),
            }
        };
        final_byte.encode(writer, protocol_version)?;
        Ok(())
    }
}
