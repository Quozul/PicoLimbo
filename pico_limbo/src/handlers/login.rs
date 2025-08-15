use crate::forwarding::check_velocity_key_integrity::{VelocityKeyIntegrity, read_velocity_key};
use crate::handlers::configuration::{send_configuration_packets, send_play_packets};
use crate::server::client_state::ClientState;
use crate::server::game_profile::GameProfile;
use crate::server::packet_handler::{PacketHandler, PacketHandlerError};
use crate::server::packet_registry::{PacketRegistry, PacketRegistryError};
use crate::server_state::ServerState;
use minecraft_packets::login::custom_query_answer_packet::CustomQueryAnswerPacket;
use minecraft_packets::login::custom_query_packet::CustomQueryPacket;
use minecraft_packets::login::game_profile_packet::GameProfilePacket;
use minecraft_packets::login::login_acknowledged_packet::LoginAcknowledgedPacket;
use minecraft_packets::login::login_state_packet::LoginStartPacket;
use minecraft_packets::login::login_success_packet::LoginSuccessPacket;
use minecraft_protocol::prelude::{BinaryReader, ProtocolVersion};
use minecraft_protocol::state::State;
use rand::Rng;

impl PacketHandler for LoginStartPacket {
    fn handle(
        &self,
        client_state: &mut ClientState,
        server_state: &ServerState,
    ) -> Result<(), PacketHandlerError> {
        if server_state.is_modern_forwarding() {
            let is_modern_forwarding_supported =
                client_state.protocol_version() >= ProtocolVersion::V1_13;
            if is_modern_forwarding_supported {
                login_start_velocity(client_state)?;
            } else {
                client_state.kick("Your client does not support modern forwarding.");
            }
        } else {
            let game_profile: GameProfile = self.into();
            fire_login_success(client_state, server_state, game_profile)?;
        }

        Ok(())
    }
}

fn login_start_velocity(client_state: &mut ClientState) -> Result<(), PacketRegistryError> {
    let message_id = {
        let mut rng = rand::rng();
        rng.random()
    };
    client_state.set_velocity_login_message_id(message_id);
    let packet = CustomQueryPacket::velocity_info_channel(message_id);
    client_state.queue_packet(PacketRegistry::CustomQuery(packet))?;
    Ok(())
}

impl PacketHandler for LoginAcknowledgedPacket {
    fn handle(
        &self,
        client_state: &mut ClientState,
        server_state: &ServerState,
    ) -> Result<(), PacketHandlerError> {
        let protocol_version = client_state.protocol_version();
        if protocol_version >= ProtocolVersion::V1_20_2 {
            client_state.set_state(State::Configuration);
            send_configuration_packets(client_state, server_state)?;
        }
        Ok(())
    }
}

impl PacketHandler for CustomQueryAnswerPacket {
    fn handle(
        &self,
        client_state: &mut ClientState,
        server_state: &ServerState,
    ) -> Result<(), PacketHandlerError> {
        let client_message_id = client_state.get_velocity_login_message_id();

        if server_state.is_modern_forwarding() && self.message_id.inner() == client_message_id {
            let secret_key = server_state
                .secret_key()
                .map_err(|_| PacketHandlerError::custom("No secret key"))?;
            let mut reader = BinaryReader::new(&self.data);
            let velocity_key = read_velocity_key(&mut reader, &secret_key);

            match velocity_key {
                VelocityKeyIntegrity::Invalid => {
                    client_state.kick("You must connect through a proxy.");
                }
                VelocityKeyIntegrity::Valid {
                    player_uuid,
                    player_name,
                } => {
                    let game_profile = GameProfile::new(&player_name, player_uuid);
                    fire_login_success(client_state, server_state, game_profile)?;
                }
            }
        }
        Ok(())
    }
}

fn fire_login_success(
    client_state: &mut ClientState,
    server_state: &ServerState,
    game_profile: GameProfile,
) -> Result<(), PacketHandlerError> {
    let protocol_version = client_state.protocol_version();

    if ProtocolVersion::V1_21_2 <= protocol_version {
        let packet = LoginSuccessPacket::new(game_profile.uuid(), game_profile.username());
        client_state.queue_packet(PacketRegistry::LoginSuccess(packet))?;
    } else {
        let packet = GameProfilePacket::new(game_profile.uuid(), game_profile.username());
        client_state.queue_packet(PacketRegistry::GameProfile(packet))?;
    }

    client_state.set_game_profile(game_profile);

    if protocol_version <= ProtocolVersion::V1_20 {
        send_play_packets(client_state, server_state)?;
    }
    Ok(())
}
