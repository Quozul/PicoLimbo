use minecraft_protocol::prelude::*;

#[derive(PacketIn, PacketOut, Clone)]
pub struct ReportIdMapping {
    pub bits_per_entry: u8,
    pub ids: LengthPaddedVec<BlocksReportId>,
}

pub type BlocksReportId = u32;
