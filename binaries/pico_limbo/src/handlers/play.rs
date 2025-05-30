use crate::ServerState;
use minecraft_packets::play::player_position::PlayerPositionPacket;
use minecraft_protocol::state::State;
use minecraft_server::client::Client;
use minecraft_server::event_handler::HandlerError;

pub async fn on_player_position(
    _state: ServerState,
    client: Client,
    _packet: PlayerPositionPacket,
) -> Result<(), HandlerError> {
    client.set_state(State::Play).await;
    client.send_keep_alive().await?;
    Ok(())
}
