mod block_state_builder;
use minecraft_protocol::prelude::{BinaryReader, BinaryReaderError, DecodePacket, ProtocolVersion};

static DATA: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/internal_mapping"));

pub fn load_internal_mapping() -> Result<InternalMapping, BinaryReaderError> {
    let mut reader = BinaryReader::new(DATA);
    InternalMapping::decode(&mut reader, ProtocolVersion::latest())
}

pub use block_state_builder::BlockStateBuilder;
pub use blocks_report_data::internal_mapping::InternalId;
pub use blocks_report_data::internal_mapping::InternalMapping;
use blocks_report_data::report_mapping::BlocksReportId;
pub use blocks_report_data::report_mapping::ReportIdMapping;
include!(concat!(env!("OUT_DIR"), "/get_blocks_reports.rs"));

pub fn get_block_report_id_mapping(protocol_version: ProtocolVersion) -> Option<ReportIdMapping> {
    get_blocks_reports(protocol_version)
}

pub fn get_block_id(
    report_mapping: &ReportIdMapping,
    internal_id: InternalId,
) -> Option<BlocksReportId> {
    report_mapping.get(&internal_id).copied()
}
