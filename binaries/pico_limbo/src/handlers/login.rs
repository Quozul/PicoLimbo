use crate::ServerState;
use crate::handlers::configuration::{send_configuration_packets, send_play_packets};
use crate::velocity::check_velocity_key_integrity::check_velocity_key_integrity;
use minecraft_packets::login::custom_query_answer_packet::CustomQueryAnswerPacket;
use minecraft_packets::login::custom_query_packet::CustomQueryPacket;
use minecraft_packets::login::game_profile_packet::GameProfilePacket;
use minecraft_packets::login::login_acknowledged_packet::LoginAcknowledgedPacket;
use minecraft_packets::login::login_disconnect_packet::LoginDisconnectPacket;
use minecraft_packets::login::login_state_packet::LoginStartPacket;
use minecraft_packets::login::login_success_packet::LoginSuccessPacket;
use minecraft_protocol::prelude::{DecodePacketField, Uuid};
use minecraft_protocol::protocol_version::ProtocolVersion;
use minecraft_protocol::state::State;
use minecraft_server::client::SharedClient;
use minecraft_server::game_profile::GameProfile;
use rand::Rng;
use tracing::info;

pub async fn on_login_start(state: ServerState, client: SharedClient, packet: LoginStartPacket) {
    if state.is_modern_forwarding() {
        login_start_velocity(client, packet).await;
    } else {
        let game_profile: GameProfile = packet.into();
        fire_login_success(client, game_profile).await;
    }
}

pub async fn on_login_acknowledged(
    _state: ServerState,
    client: SharedClient,
    _packet: LoginAcknowledgedPacket,
) {
    let mut client = client.lock().await;
    if client.protocol_version() >= ProtocolVersion::V1_20_2 {
        client.update_state(State::Configuration);
    }
}

pub async fn on_custom_query_answer(
    state: ServerState,
    client: SharedClient,
    packet: CustomQueryAnswerPacket,
) {
    let client_message_id = {
        let client = client.lock().await;
        client.get_velocity_login_message_id()
    };

    if state.is_modern_forwarding() && packet.message_id.value() == client_message_id {
        let buf = &packet.data;
        let mut index = 0;
        let is_valid =
            check_velocity_key_integrity(buf, state.secret_key(), &mut index).unwrap_or_default();
        if is_valid {
            let _address = String::decode(buf, &mut index).unwrap_or_default();
            let player_uuid = Uuid::decode(buf, &mut index).unwrap_or_default();
            let player_name = String::decode(buf, &mut index).unwrap_or_default();

            let game_profile = GameProfile::new(player_name, player_uuid);
            fire_login_success(client, game_profile).await;
        } else {
            let packet = LoginDisconnectPacket::text("You must connect through a proxy.")
                .unwrap_or(LoginDisconnectPacket::default());
            let mut client = client.lock().await;
            client.send_packet(packet).await;
        }
    }
}

async fn login_start_velocity(client: SharedClient, _packet: LoginStartPacket) {
    let mut client = client.lock().await;

    let packet = {
        let mut rng = rand::rng();
        let message_id = rng.random();
        client.set_velocity_login_message_id(message_id);
        CustomQueryPacket::velocity_info_channel(message_id)
    };
    client.send_packet(packet).await;
}

async fn fire_login_success(client: SharedClient, game_profile: GameProfile) {
    let mut client = client.lock().await;

    if ProtocolVersion::V1_21_2 <= client.protocol_version() {
        let packet = LoginSuccessPacket::new(game_profile.uuid(), game_profile.username());
        client.send_packet(packet).await;
    } else {
        let packet = GameProfilePacket::new(game_profile.uuid(), game_profile.username());
        client.send_packet(packet).await;
    }

    client.set_game_profile(game_profile.clone());
    info!(
        "UUID of player {} is {}",
        game_profile.username(),
        game_profile.uuid()
    );

    if ProtocolVersion::V1_20_2 <= client.protocol_version() {
        send_configuration_packets(client).await;
    } else {
        send_play_packets(client).await;
    }
}
