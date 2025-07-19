use crate::forwarding::check_bungee_cord::check_bungee_cord;
use crate::server::client::Client;
use crate::server::event_handler::HandlerError;
use crate::server_state::ServerState;
use minecraft_packets::handshaking::handshake_packet::HandshakePacket;
use minecraft_protocol::protocol_version::ProtocolVersion;
use minecraft_protocol::state::State;
use thiserror::Error;

pub async fn on_handshake(
    state: ServerState,
    client: Client,
    packet: HandshakePacket,
) -> Result<(), HandlerError> {
    client.set_protocol_version(packet.get_protocol()).await;

    if let Ok(next_state) = packet.get_next_state() {
        client.set_state(next_state).await;

        if client.current_state().await == State::Login
            && !check_bungee_cord(state, packet.hostname)?
        {
            client.kick("You must connect through a proxy.").await?;
        }
    }

    Ok(())
}

#[derive(Error, Debug)]
#[error("unknown state {0}")]
struct UnknownStateError(i32);

trait GetStateProtocol {
    fn get_next_state(&self) -> Result<State, UnknownStateError>;
    fn get_protocol(&self) -> ProtocolVersion;
}

impl GetStateProtocol for HandshakePacket {
    fn get_next_state(&self) -> Result<State, UnknownStateError> {
        let state = self.next_state.value();
        match state {
            1 => Ok(State::Status),
            2 => Ok(State::Login),
            3 => Ok(State::Transfer),
            _ => Err(UnknownStateError(state)),
        }
    }

    fn get_protocol(&self) -> ProtocolVersion {
        if self.protocol.value() == -1 {
            ProtocolVersion::Any
        } else {
            ProtocolVersion::from(self.protocol.value())
        }
    }
}
