use minecraft_protocol::prelude::*;

#[derive(PacketOut)]
pub struct UpdateLightPacketV1_16 {
    chunk_x: VarInt,
    chunk_z: VarInt,
    trust_edges: bool,
    sky_light_mask: VarInt,
    block_light_mask: VarInt,
    empty_sky_light_mask: VarInt,
    empty_block_light_mask: VarInt,
    sky_light_arrays: Vec<LengthPaddedVec<u8>>,
    block_light_arrays: Vec<LengthPaddedVec<u8>>,
}

#[derive(PacketOut)]
pub struct UpdateLightPacketV1_17 {
    chunk_x: VarInt,
    chunk_z: VarInt,
    trust_edges: bool,
    sky_light_mask: BitSet,
    block_light_mask: BitSet,
    empty_sky_light_mask: BitSet,
    empty_block_light_mask: BitSet,
    sky_light_arrays: LengthPaddedVec<LengthPaddedVec<u8>>,
    block_light_arrays: LengthPaddedVec<LengthPaddedVec<u8>>,
}

pub enum UpdateLightPacket {
    V1_16(UpdateLightPacketV1_16),
    V1_17(UpdateLightPacketV1_17),
}

impl EncodePacket for UpdateLightPacket {
    fn encode(
        &self,
        writer: &mut BinaryWriter,
        protocol_version: ProtocolVersion,
    ) -> Result<(), BinaryWriterError> {
        match self {
            Self::V1_16(packet) => packet.encode(writer, protocol_version),
            Self::V1_17(packet) => packet.encode(writer, protocol_version),
        }
    }
}

impl UpdateLightPacketV1_16 {
    pub fn full_bright(chunk_x: i32, chunk_z: i32) -> Self {
        Self {
            chunk_x: VarInt::new(chunk_x),
            chunk_z: VarInt::new(chunk_z),
            trust_edges: true,
            sky_light_mask: VarInt::new(0x3ffff),
            block_light_mask: VarInt::new(0),
            empty_sky_light_mask: VarInt::new(0),
            empty_block_light_mask: VarInt::new(0x3ffff),
            sky_light_arrays: vec![LengthPaddedVec::new(vec![0xFF; 2048]); 18],
            block_light_arrays: vec![],
        }
    }
}

impl UpdateLightPacketV1_17 {
    pub fn full_bright(chunk_x: i32, chunk_z: i32) -> Self {
        Self {
            chunk_x: VarInt::new(chunk_x),
            chunk_z: VarInt::new(chunk_z),
            trust_edges: true,
            sky_light_mask: BitSet::new(vec![0x3ffff]),
            block_light_mask: BitSet::new(vec![0]),
            empty_sky_light_mask: BitSet::new(vec![0]),
            empty_block_light_mask: BitSet::new(vec![0x3ffff]),
            sky_light_arrays: LengthPaddedVec::new(vec![
                LengthPaddedVec::new(vec![0xFF; 2048]);
                18
            ]),
            block_light_arrays: LengthPaddedVec::new(vec![]),
        }
    }
}

impl UpdateLightPacket {
    pub fn full_bright(chunk_x: i32, chunk_z: i32, protocol_version: ProtocolVersion) -> Self {
        if protocol_version.is_before_inclusive(ProtocolVersion::V1_16_4) {
            Self::V1_16(UpdateLightPacketV1_16::full_bright(chunk_x, chunk_z))
        } else {
            Self::V1_17(UpdateLightPacketV1_17::full_bright(chunk_x, chunk_z))
        }
    }
}
