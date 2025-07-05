use crate::play::data::chunk_section::ChunkSection;
use minecraft_protocol::prelude::*;
use minecraft_protocol::protocol_version::ProtocolVersion;
use pico_structures::prelude::Structure;
use thiserror::Error;
use tracing::error;

#[derive(Debug)]
pub struct ChunkData {
    height_maps: Nbt,
    v1_21_5_height_maps: LengthPaddedVec<HeightMap>,

    /// Biome IDs, ordered by x then z then y, in 4×4×4 blocks.
    /// Up until 1.17.5 included
    biomes: LengthPaddedVec<VarInt>,

    /// Size of Data in bytes!
    /// LengthPaddedVec prefixes with the number of elements!
    data: Vec<ChunkSection>,
    block_entities: LengthPaddedVec<BlockEntity>,
}

#[derive(Debug, Error)]
enum HeightMapError {
    #[error(transparent)]
    Infallible(#[from] std::convert::Infallible),
    #[error(transparent)]
    Vec(#[from] LengthPaddedVecEncodeError),
}

#[derive(Debug)]
struct HeightMap {
    /// 1: WORLD_SURFACE
    /// All blocks other than air, cave air and void air. To determine if a beacon beam is obstructed.
    /// 4: MOTION_BLOCKING
    /// "Solid" blocks, except bamboo saplings and cactuses; fluids. To determine where to display rain and snow.
    /// 5: MOTION_BLOCKING_NO_LEAVES
    /// Same as MOTION_BLOCKING, excluding leaf blocks.
    height_map_type: VarInt,
    data: LengthPaddedVec<i64>,
}

impl EncodePacketField for HeightMap {
    type Error = HeightMapError;

    fn encode(&self, bytes: &mut Vec<u8>, protocol_version: u32) -> Result<(), Self::Error> {
        self.height_map_type.encode(bytes, protocol_version)?;
        self.data.encode(bytes, protocol_version)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct BlockEntity {
    // TODO: Implement BlockEntity
}

impl EncodePacketField for BlockEntity {
    type Error = std::convert::Infallible;

    fn encode(&self, _bytes: &mut Vec<u8>, _protocol_version: u32) -> Result<(), Self::Error> {
        // Nothing to encode
        Ok(())
    }
}

impl ChunkData {
    pub fn void(biome_index: i32) -> Self {
        let long_array_tag = Nbt::LongArray {
            name: Some("MOTION_BLOCKING".to_string()),
            value: vec![0; 37],
        };
        let root_tag = Nbt::Compound {
            name: None,
            value: vec![long_array_tag],
        };

        Self {
            height_maps: root_tag,
            v1_21_5_height_maps: LengthPaddedVec::new(vec![HeightMap {
                height_map_type: VarInt::new(4), // Motionblock type
                data: LengthPaddedVec::new(vec![0; 37]),
            }]),
            biomes: vec![VarInt::new(127); 1024].into(),
            data: vec![ChunkSection::void(biome_index); 24],
            block_entities: Vec::new().into(),
        }
    }

    pub fn all_stone(structure: &Structure, void_biome_index: i32) -> Self {
        let long_array_tag = Nbt::LongArray {
            name: Some("MOTION_BLOCKING".to_string()),
            value: vec![0; 37],
        };
        let root_tag = Nbt::Compound {
            name: None,
            value: vec![long_array_tag],
        };

        let mut data = Vec::new();

        for i in 0..24 {
            let section = ChunkSection::from_structure(structure, void_biome_index);
            data.push(section);
        }

        Self {
            height_maps: root_tag,
            v1_21_5_height_maps: LengthPaddedVec::new(vec![HeightMap {
                height_map_type: VarInt::new(4), // Motionblock type
                data: LengthPaddedVec::new(vec![0; 37]),
            }]),
            data,
            biomes: LengthPaddedVec::default(),
            block_entities: Vec::new().into(),
        }
    }
}

#[derive(Debug, Error)]
pub enum ChunkDataError {
    #[error(transparent)]
    Nbt(#[from] NbtEncodeError),
    #[error(transparent)]
    Vec(#[from] LengthPaddedVecEncodeError),
    #[error(transparent)]
    VecEncodeError(#[from] VecEncodeError),
    #[error(transparent)]
    Infallible(#[from] std::convert::Infallible),
}

impl EncodePacketField for ChunkData {
    type Error = ChunkDataError;

    fn encode(&self, bytes: &mut Vec<u8>, protocol_version: u32) -> Result<(), Self::Error> {
        // Encode height maps, version agnostic
        if protocol_version >= ProtocolVersion::V1_21_5.version_number() {
            self.v1_21_5_height_maps.encode(bytes, protocol_version)?;
        } else {
            self.height_maps.encode(bytes, protocol_version)?;
        }

        // Encode biomes
        if protocol_version <= ProtocolVersion::V1_17_1.version_number() {
            self.biomes.encode(bytes, protocol_version)?;
        }

        // Encode data
        let mut encoded_data = Vec::<u8>::new();
        self.data.encode(&mut encoded_data, protocol_version)?;

        let mut chunk_sections_payload = Vec::<u8>::new();
        self.data
            .encode(&mut chunk_sections_payload, protocol_version)?;

        let payload_size = VarInt::new(chunk_sections_payload.len() as i32);
        payload_size.encode(bytes, protocol_version)?;

        bytes.extend_from_slice(&chunk_sections_payload);

        // Encode block entities
        self.block_entities.encode(bytes, protocol_version)?;

        Ok(())
    }
}
