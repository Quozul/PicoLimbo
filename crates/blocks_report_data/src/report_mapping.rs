use minecraft_protocol::prelude::LengthPaddedVec;
use protocol_version::protocol_version::ProtocolVersion;

pub type ReportIdMapping = LengthPaddedVec<BlocksReportId>;

pub struct ReportMapping {
    pub protocol_version: ProtocolVersion,
    pub mapping: ReportIdMapping,
}

pub type BlocksReportId = u16;
