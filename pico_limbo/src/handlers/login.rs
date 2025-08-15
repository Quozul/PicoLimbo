use crate::forwarding::check_velocity_key_integrity::{VelocityKeyIntegrity, read_velocity_key};
use crate::handlers::configuration::{send_configuration_packets, send_play_packets};
use crate::server::client::Client;
use crate::server::event_handler::HandlerError;
use crate::server::game_profile::GameProfile;
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
use tracing::info;

pub async fn on_login_start(
    state: ServerState,
    client: Client,
    packet: LoginStartPacket,
) -> Result<(), HandlerError> {
    if state.is_modern_forwarding() {
        let is_modern_forwarding_supported =
            client.protocol_version().await >= ProtocolVersion::V1_13;
        if is_modern_forwarding_supported {
            login_start_velocity(client).await?;
        } else {
            client
                .kick("Your client does not support modern forwarding.")
                .await?;
        }
    } else {
        let game_profile: GameProfile = packet.into();
        fire_login_success(client, game_profile, state).await?;
    }

    Ok(())
}

pub async fn on_login_acknowledged(
    state: ServerState,
    client: Client,
    _packet: LoginAcknowledgedPacket,
) -> Result<(), HandlerError> {
    let protocol_version = client.protocol_version().await;
    if protocol_version >= ProtocolVersion::V1_20_2 {
        client.set_state(State::Configuration).await;
        send_configuration_packets(client, state).await?;
    }
    Ok(())
}

pub async fn on_custom_query_answer(
    state: ServerState,
    client: Client,
    packet: CustomQueryAnswerPacket,
) -> Result<(), HandlerError> {
    let client_message_id = client.get_velocity_login_message_id().await;

    if state.is_modern_forwarding() && packet.message_id.inner() == client_message_id {
        let secret_key = state
            .secret_key()
            .map_err(|_| HandlerError::custom("No secret key"))?;
        let mut reader = BinaryReader::new(&packet.data);
        let velocity_key = read_velocity_key(&mut reader, &secret_key);

        match velocity_key {
            VelocityKeyIntegrity::Invalid => {
                client.kick("You must connect through a proxy.").await?;
            }
            VelocityKeyIntegrity::Valid {
                player_uuid,
                player_name,
            } => {
                let game_profile = GameProfile::new(&player_name, player_uuid);
                fire_login_success(client, game_profile, state).await?;
            }
        }
    }
    Ok(())
}

async fn login_start_velocity(client: Client) -> Result<(), HandlerError> {
    let message_id = {
        let mut rng = rand::rng();
        rng.random()
    };
    client.set_velocity_login_message_id(message_id).await;
    let packet = CustomQueryPacket::velocity_info_channel(message_id);
    client.send_packet(packet).await?;
    Ok(())
}

async fn fire_login_success(
    client: Client,
    game_profile: GameProfile,
    state: ServerState,
) -> Result<(), HandlerError> {
    let protocol_version = client.protocol_version().await;

    if ProtocolVersion::V1_21_2 <= protocol_version {
        let packet = LoginSuccessPacket::new(game_profile.uuid(), game_profile.username());
        client.send_packet(packet).await?;
    } else {
        let packet = GameProfilePacket::new(game_profile.uuid(), game_profile.username());
        client.send_packet(packet).await?;
    }

    client.set_game_profile(game_profile.clone()).await;
    info!(
        "UUID of player {} is {}",
        game_profile.username(),
        game_profile.uuid()
    );

    if protocol_version <= ProtocolVersion::V1_20 {
        send_play_packets(client, state).await?;
    }
    Ok(())
}
