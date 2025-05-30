use crate::ServerState;
use minecraft_packets::play::player_position::PlayerPositionPacket;
use minecraft_protocol::state::State;
use minecraft_server::client::Client;

pub async fn on_player_position(
    _state: ServerState,
    client: Client,
    _packet: PlayerPositionPacket,
) {
    client.set_state(State::Play).await;
    client.send_keep_alive().await;
}
