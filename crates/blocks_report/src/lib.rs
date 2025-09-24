use minecraft_protocol::prelude::{BinaryReader, BinaryReaderError, DecodePacket, ProtocolVersion};
use thiserror::Error;

pub use blocks_report_data::{
    block_state_builder::BlockStateLookup,
    internal_mapping::{InternalId, InternalMapping},
    report_mapping::{BlocksReportId, ReportIdMapping},
};

include!(concat!(env!("OUT_DIR"), "/get_blocks_reports.rs"));

static INTERNAL_MAPPING_DATA: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/internal_mapping"));

#[derive(Debug, Error)]
pub enum BlockReportIdMappingError {
    #[error("Failed to read binary data: {0}")]
    BinaryReader(#[from] BinaryReaderError),
    #[error("Protocol version {0} is not supported")]
    UnsupportedVersion(ProtocolVersion),
}

pub fn load_internal_mapping() -> Result<InternalMapping, BinaryReaderError> {
    let mut reader = BinaryReader::new(INTERNAL_MAPPING_DATA);
    InternalMapping::decode(&mut reader, ProtocolVersion::latest())
}

pub fn get_block_report_id_mapping(
    protocol_version: ProtocolVersion,
) -> Result<Vec<BlocksReportId>, BlockReportIdMappingError> {
    Ok(get_blocks_reports(protocol_version)?.into_inner())
}

pub fn get_block_id(
    report_mapping: &[BlocksReportId],
    internal_id: InternalId,
) -> Option<BlocksReportId> {
    report_mapping.get(internal_id as usize).copied()
}
