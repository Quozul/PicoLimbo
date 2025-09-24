use minecraft_protocol::prelude::LengthPaddedVec;

pub type ReportIdMapping = LengthPaddedVec<BlocksReportId>;

pub type BlocksReportId = u16;
