use crate::internal_mapping::InternalId;
use protocol_version::protocol_version::ProtocolVersion;
use std::collections::HashMap;

pub type ReportIdMapping = HashMap<InternalId, BlocksReportId>;

pub struct ReportMapping {
    pub protocol_version: ProtocolVersion,
    pub mapping: ReportIdMapping,
}

pub type BlocksReportId = u16;
