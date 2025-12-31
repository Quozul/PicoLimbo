use crate::handlers::play::set_player_position_and_rotation::teleport_player_to_spawn;
use crate::server::batch::Batch;
use crate::server::client_state::ClientState;
use crate::server::packet_handler::{PacketHandler, PacketHandlerError};
use crate::server::packet_registry::PacketRegistry;
use crate::server_state::ServerState;
use minecraft_packets::play::chat_command_packet::ChatCommandPacket;
use minecraft_packets::play::chat_message_packet::ChatMessagePacket;
use minecraft_packets::play::client_bound_player_abilities_packet::ClientBoundPlayerAbilitiesPacket;
use thiserror::Error;
use tracing::info;

impl PacketHandler for ChatCommandPacket {
    fn handle(
        &self,
        client_state: &mut ClientState,
        server_state: &ServerState,
    ) -> Result<Batch<PacketRegistry>, PacketHandlerError> {
        let mut batch = Batch::new();
        run_command(client_state, server_state, self.get_command(), &mut batch);
        Ok(batch)
    }
}

impl PacketHandler for ChatMessagePacket {
    fn handle(
        &self,
        client_state: &mut ClientState,
        server_state: &ServerState,
    ) -> Result<Batch<PacketRegistry>, PacketHandlerError> {
        let mut batch = Batch::new();
        if let Some(command) = self.get_command() {
            run_command(client_state, server_state, command, &mut batch);
        } else {
            info!("<{}> {}", client_state.get_username(), self.get_message());
        }
        Ok(batch)
    }
}

fn run_command(
    client_state: &mut ClientState,
    server_state: &ServerState,
    command: &str,
    batch: &mut Batch<PacketRegistry>,
) {
    info!(
        "{} issued server command: /{}",
        client_state.get_username(),
        command
    );

    if let Ok(parsed_command) = Command::parse(command) {
        match parsed_command {
            Command::Spawn => {
                teleport_player_to_spawn(client_state, server_state, batch);
            }
            Command::Fly => {
                let allow_flying = !client_state.is_flight_allowed();
                let flying = allow_flying && client_state.is_flying();
                let packet = ClientBoundPlayerAbilitiesPacket::builder()
                    .allow_flying(allow_flying)
                    .flying(flying)
                    .flying_speed(client_state.get_flying_speed())
                    .build();
                batch.queue(|| PacketRegistry::ClientBoundPlayerAbilities(packet));
                client_state.set_is_flight_allowed(allow_flying);
                client_state.set_is_flying(allow_flying);
            }
            Command::FlySpeed(speed) => {
                let packet = ClientBoundPlayerAbilitiesPacket::builder()
                    .allow_flying(client_state.is_flight_allowed())
                    .flying(client_state.is_flying())
                    .flying_speed(speed)
                    .build();
                batch.queue(|| PacketRegistry::ClientBoundPlayerAbilities(packet));
                client_state.set_flying_speed(speed);
            }
        }
    }
}

#[derive(Debug, Error)]
pub enum ParseCommandError {
    #[error("empty command")]
    Empty,
    #[error("unknown command")]
    Unknown,
    #[error("invalid speed value")]
    InvalidSpeed(#[from] std::num::ParseFloatError),
}

enum Command {
    Spawn,
    Fly,
    FlySpeed(f32),
}

impl Command {
    pub fn parse(input: &str) -> Result<Self, ParseCommandError> {
        let mut parts = input.split_whitespace();
        let cmd = parts.next().ok_or(ParseCommandError::Empty)?;
        if cmd == "spawn" {
            Ok(Self::Spawn)
        } else if cmd == "fly" {
            Ok(Self::Fly)
        } else if cmd == "flyspeed" {
            let speed_str = parts.next().unwrap_or("0.5");
            let speed = speed_str.parse::<f32>()?.clamp(0.0, 1.0);
            Ok(Self::FlySpeed(speed))
        } else {
            Err(ParseCommandError::Unknown)
        }
    }
}
