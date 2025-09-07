use minecraft_protocol::prelude::*;

#[derive(PacketOut)]
pub struct UpdateTimePacket {
    pub world_age: i64,
    pub time_of_day: i64,
    #[pvn(768..)]
    pub time_of_day_increasing: bool,
}

impl UpdateTimePacket {
    pub fn new(world_age: i64, time_of_day: i64, time_of_day_increasing: bool) -> Self {
        Self {
            world_age,
            time_of_day,
            time_of_day_increasing,
        }
    }
}
