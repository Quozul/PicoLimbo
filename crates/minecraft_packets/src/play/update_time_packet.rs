use minecraft_protocol::prelude::*;
use std::collections::HashMap;

#[derive(Eq, PartialEq, Hash)]
enum Holder {
    Reference { id: VarInt },
}

impl Holder {
    pub fn reference(id: i32) -> Self {
        Self::Reference {
            id: VarInt::new(id),
        }
    }
}

impl EncodePacket for Holder {
    fn encode(
        &self,
        writer: &mut BinaryWriter,
        protocol_version: ProtocolVersion,
    ) -> Result<(), BinaryWriterError> {
        match self {
            Self::Reference { id } => id.encode(writer, protocol_version),
        }
    }
}

#[derive(PacketOut)]
pub struct UpdateTimePacket {
    game_time: i64,
    #[pvn(..775)]
    time_of_day: i64,
    #[pvn(768..775)]
    time_of_day_increasing: bool,
    #[pvn(775..)]
    clock_updates: HashMap<Holder, ClockNetworkState>,
}

#[derive(PacketOut)]
struct ClockNetworkState {
    total_ticks: VarLong,
    partial_tick: f32,
    rate: f32,
}

impl UpdateTimePacket {
    pub fn new(world_age: i64, time_of_day_increasing: bool) -> Self {
        let rate = if time_of_day_increasing { 1.0 } else { 0.0 };
        let mut clock_updates = HashMap::new();
        let state = ClockNetworkState {
            total_ticks: VarLong::new(world_age),
            partial_tick: 0.0,
            rate,
        };
        clock_updates.insert(Holder::reference(0), state);
        Self {
            game_time: world_age,
            time_of_day: world_age,
            time_of_day_increasing,
            clock_updates,
        }
    }
}
