use minecraft_protocol::prelude::*;

/// This packet was introduced in Minecraft version 1.20.3.
/// Used for a wide variety of game events, from weather to bed use to game mode to demo messages.
#[derive(PacketOut)]
pub struct GameEventPacket {
    /// See the GameEvent below.
    event: u8,
    /// Depends on Event.
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
