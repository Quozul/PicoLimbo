use data_types::prelude::EncodePacketField;
use macros::{PacketOut, packet_id};

#[derive(Debug, PacketOut)]
#[packet_id(0x23)]
pub struct GameEventPacket {
    event: u8,
    value: f32,
}

impl GameEventPacket {
    pub fn start_waiting_for_chunks(value: f32) -> Self {
        Self {
            event: GameEvent::StartWaitingForChunks.get_event_id(),
            value,
        }
    }
}

enum GameEvent {
    StartWaitingForChunks,
}

impl GameEvent {
    fn get_event_id(&self) -> u8 {
        match self {
            GameEvent::StartWaitingForChunks => 13,
        }
    }
}
