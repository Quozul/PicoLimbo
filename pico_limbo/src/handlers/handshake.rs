use crate::forwarding::check_bungee_cord::check_bungee_cord;
use crate::forwarding::forwarding_result::LegacyForwardingResult;
use crate::kick_messages::PROXY_REQUIRED_KICK_MESSAGE;
use crate::server::batch::Batch;
use crate::server::client_state::ClientState;
use crate::server::game_profile::GameProfile;
use crate::server::packet_handler::{PacketHandler, PacketHandlerError};
use crate::server_state::ServerState;
use minecraft_packets::handshaking::handshake_packet::HandshakePacket;
use minecraft_protocol::prelude::{ProtocolVersion, State};
use thiserror::Error;
use tracing::debug;

impl PacketHandler for HandshakePacket {
    fn handle(
        &self,
        client_state: &mut ClientState,
        server_state: &ServerState,
    ) -> Result<Batch, PacketHandlerError> {
        let mut batch = Batch::new();
        let protocol_version = self.get_protocol(server_state.allow_unsupported_versions());
        debug!("Client requested protocol version {}", protocol_version);
        client_state.set_protocol_version(protocol_version);

        self.get_next_state().map_or_else(
            |err| {
                Err(PacketHandlerError::invalid_state(&format!(
                    "Unsupported next state {}",
                    err.0
                )))
            },
            |next_state| {
                batch.queue_both_state_change(next_state);

                match next_state {
                    State::Status => {
                        if server_state.reply_to_status() {
                            Ok(batch)
                        } else {
                            Err(PacketHandlerError::disconnect("Ignoring status request"))
                        }
                    }
                    State::Login => {
                        begin_login(client_state, server_state, &self.hostname)?;
                        Ok(batch)
                    }
                    State::Transfer => {
                        if server_state.accept_transfers() {
                            batch.queue_both_state_change(State::Login);
                            begin_login(client_state, server_state, &self.hostname)?;
                            Ok(batch)
                        } else {
                            Err(PacketHandlerError::disconnect("Transfers disabled"))
                        }
                    }
                    state => Err(PacketHandlerError::invalid_state(&format!(
                        "Invalid intention {state}"
                    ))),
                }
            },
        )
    }
}

fn begin_login(
    client_state: &mut ClientState,
    server_state: &ServerState,
    hostname: &str,
) -> Result<(), PacketHandlerError> {
    if client_state.protocol_version().is_unsupported() {
        return Err(PacketHandlerError::invalid_state(&format!(
            "Unsupported protocol version {}",
            client_state.protocol_version()
        )));
    }

    let (clean_hostname, floodgate_data) = server_state
        .floodgate()
        .parse_hostname(hostname)
        .map_err(|error| PacketHandlerError::invalid_state(&error))?;

    if let Some(data) = floodgate_data {
        let (username, uuid) = server_state
            .floodgate()
            .game_profile(&data)
            .map_err(|error| PacketHandlerError::invalid_state(&error))?;
        client_state.replace_game_profile(GameProfile::new(&username, uuid, None));
        return Ok(());
    }

    let forwarding_result = check_bungee_cord(server_state, &clean_hostname);
    match forwarding_result {
        LegacyForwardingResult::Invalid => {
            client_state.kick(PROXY_REQUIRED_KICK_MESSAGE);
            Err(PacketHandlerError::invalid_state(
                PROXY_REQUIRED_KICK_MESSAGE,
            ))
        }
        LegacyForwardingResult::Anonymous {
            player_uuid,
            textures,
        } => {
            let game_profile = GameProfile::anonymous(player_uuid, textures);
            client_state.set_game_profile(game_profile);

            Ok(())
        }
        LegacyForwardingResult::NoForwarding => Ok(()),
    }
}
