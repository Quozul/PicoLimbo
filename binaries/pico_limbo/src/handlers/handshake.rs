use crate::ServerState;
use minecraft_packets::handshaking::handshake_packet::HandshakePacket;
use minecraft_protocol::protocol_version::ProtocolVersion;
use minecraft_protocol::state::State;
use minecraft_server::client::Client;
use thiserror::Error;

pub async fn on_handshake(_state: ServerState, client: Client, packet: HandshakePacket) {
    client.set_protocol_version(packet.get_protocol()).await;

    if let Ok(state) = packet.get_next_state() {
        client.set_state(state).await;
    }
}

#[derive(Error, Debug)]
#[error("unknown state {0}")]
pub struct UnknownStateError(i32);

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
        ProtocolVersion::from(self.protocol.value())
    }
}
