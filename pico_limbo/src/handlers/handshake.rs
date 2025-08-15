use crate::forwarding::check_bungee_cord::check_bungee_cord;
use crate::server::client_state::ClientState;
use crate::server::packet_handler::{PacketHandler, PacketHandlerError};
use crate::server_state::ServerState;
use minecraft_packets::handshaking::handshake_packet::HandshakePacket;
use minecraft_protocol::prelude::ProtocolVersion;
use minecraft_protocol::state::State;
use thiserror::Error;

impl PacketHandler for HandshakePacket {
    fn handle(
        &self,
        client_state: &mut ClientState,
        server_state: &ServerState,
    ) -> Result<(), PacketHandlerError> {
        client_state.set_protocol_version(self.get_protocol());

        if let Ok(next_state) = self.get_next_state() {
            client_state.set_state(next_state);

            if client_state.state() == State::Login
                && !check_bungee_cord(server_state, &self.hostname)?
            {
                client_state.kick("You must connect through a proxy.");
            }
        }

        Ok(())
    }
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
        let state = self.next_state.inner();
        match state {
            1 => Ok(State::Status),
            2 => Ok(State::Login),
            3 => Ok(State::Transfer),
            _ => Err(UnknownStateError(state)),
        }
    }

    fn get_protocol(&self) -> ProtocolVersion {
        if self.protocol.inner() == -1 {
            ProtocolVersion::Any
        } else {
            ProtocolVersion::from(self.protocol.inner())
        }
    }
}
