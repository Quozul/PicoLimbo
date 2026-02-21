use crate::server::batch::Batch;
use crate::server::client_state::ClientState;
use crate::server::packet_handler::{PacketHandler, PacketHandlerError};
use crate::server_state::ServerState;
use macros::PacketReport;
use minecraft_packets::configuration::acknowledge_finish_configuration_packet::AcknowledgeConfigurationPacket;
use minecraft_packets::configuration::client_bound_known_packs_packet::ClientBoundKnownPacksPacket;
use minecraft_packets::configuration::configuration_client_bound_plugin_message_packet::ConfigurationClientBoundPluginMessagePacket;
use minecraft_packets::configuration::finish_configuration_packet::FinishConfigurationPacket;
use minecraft_packets::configuration::registry_data_packet::RegistryDataPacket;
use minecraft_packets::handshaking::handshake_packet::HandshakePacket;
use minecraft_packets::login::custom_query_answer_packet::CustomQueryAnswerPacket;
use minecraft_packets::login::custom_query_packet::CustomQueryPacket;
use minecraft_packets::login::game_profile_packet::GameProfilePacket;
use minecraft_packets::login::login_acknowledged_packet::LoginAcknowledgedPacket;
use minecraft_packets::login::login_disconnect_packet::LoginDisconnectPacket;
use minecraft_packets::login::login_state_packet::LoginStartPacket;
use minecraft_packets::login::login_success_packet::LoginSuccessPacket;
use minecraft_packets::login::set_compression_packet::SetCompressionPacket;
use minecraft_packets::play::boss_bar_packet::BossBarPacket;
use minecraft_packets::play::chat_command_packet::ChatCommandPacket;
use minecraft_packets::play::chat_message_packet::ChatMessagePacket;
use minecraft_packets::play::chunk_data_and_update_light_packet::ChunkDataAndUpdateLightPacket;
use minecraft_packets::play::client_bound_keep_alive_packet::ClientBoundKeepAlivePacket;
use minecraft_packets::play::client_bound_player_abilities_packet::ClientBoundPlayerAbilitiesPacket;
use minecraft_packets::play::client_bound_plugin_message_packet::PlayClientBoundPluginMessagePacket;
use minecraft_packets::play::commands_packet::CommandsPacket;
use minecraft_packets::play::disconnect_packet::DisconnectPacket;
use minecraft_packets::play::game_event_packet::GameEventPacket;
use minecraft_packets::play::legacy_chat_message_packet::LegacyChatMessagePacket;
use minecraft_packets::play::legacy_set_title_packet::LegacySetTitlePacket;
use minecraft_packets::play::login_packet::LoginPacket;
use minecraft_packets::play::pick_item_from_block_packet::PickItemFromBlockPacket;
use minecraft_packets::play::player_info_update_packet::PlayerInfoUpdatePacket;
use minecraft_packets::play::server_bound_player_abilities_packet::ServerBoundPlayerAbilitiesPacket;
use minecraft_packets::play::set_action_bar_text_packet::SetActionBarTextPacket;
use minecraft_packets::play::set_chunk_cache_center_packet::SetCenterChunkPacket;
use minecraft_packets::play::set_default_spawn_position_packet::SetDefaultSpawnPositionPacket;
use minecraft_packets::play::set_entity_data_packet::SetEntityMetadataPacket;
use minecraft_packets::play::set_player_position_and_rotation_packet::SetPlayerPositionAndRotationPacket;
use minecraft_packets::play::set_player_position_packet::SetPlayerPositionPacket;
use minecraft_packets::play::set_title_text_packet::SetTitleTextPacket;
use minecraft_packets::play::set_titles_animation::SetTitlesAnimationPacket;
use minecraft_packets::play::synchronize_player_position_packet::SynchronizePlayerPositionPacket;
use minecraft_packets::play::system_chat_message_packet::SystemChatMessagePacket;
use minecraft_packets::play::tab_list_packet::TabListPacket;
use minecraft_packets::play::transfer_packet::TransferPacket;
use minecraft_packets::play::update_time_packet::UpdateTimePacket;
use minecraft_packets::status::ping_request_packet::PingRequestPacket;
use minecraft_packets::status::ping_response_packet::PongResponsePacket;
use minecraft_packets::status::status_request_packet::StatusRequestPacket;
use minecraft_packets::status::status_response_packet::StatusResponsePacket;
use minecraft_protocol::prelude::*;
use net::raw_packet::RawPacket;

#[derive(PacketReport)]
pub enum PacketRegistry {
    // Handshake packets
    #[protocol_id(
        state = "handshake",
        bound = "serverbound",
        name = "minecraft:intention"
    )]
    Handshake(HandshakePacket),

    // Status packets
    #[protocol_id(
        state = "status",
        bound = "serverbound",
        name = "minecraft:status_request"
    )]
    StatusRequest(StatusRequestPacket),

    #[protocol_id(
        state = "status",
        bound = "clientbound",
        name = "minecraft:status_response"
    )]
    StatusResponse(StatusResponsePacket),

    #[protocol_id(
        state = "status",
        bound = "serverbound",
        name = "minecraft:ping_request"
    )]
    PingRequest(PingRequestPacket),

    #[protocol_id(
        state = "status",
        bound = "clientbound",
        name = "minecraft:pong_response"
    )]
    PongResponse(PongResponsePacket),

    // Login packets
    #[protocol_id(state = "login", bound = "serverbound", name = "minecraft:hello")]
    LoginStart(LoginStartPacket),

    #[protocol_id(
        state = "login",
        bound = "serverbound",
        name = "minecraft:login_acknowledged"
    )]
    LoginAcknowledged(LoginAcknowledgedPacket),

    #[protocol_id(
        state = "login",
        bound = "serverbound",
        name = "minecraft:custom_query_answer"
    )]
    CustomQueryAnswer(CustomQueryAnswerPacket),

    #[protocol_id(
        state = "login",
        bound = "clientbound",
        name = "minecraft:custom_query"
    )]
    CustomQuery(CustomQueryPacket),

    #[protocol_id(
        state = "login",
        bound = "clientbound",
        name = "minecraft:login_finished"
    )]
    LoginSuccess(LoginSuccessPacket),

    #[protocol_id(
        state = "login",
        bound = "clientbound",
        name = "minecraft:game_profile"
    )]
    GameProfile(GameProfilePacket),

    #[protocol_id(
        state = "login",
        bound = "clientbound",
        name = "minecraft:login_disconnect"
    )]
    LoginDisconnect(LoginDisconnectPacket),

    #[protocol_id(
        state = "login",
        bound = "clientbound",
        name = "minecraft:login_compression"
    )]
    SetCompression(SetCompressionPacket),

    // Configuration packets
    #[protocol_id(
        state = "configuration",
        bound = "serverbound",
        name = "minecraft:finish_configuration"
    )]
    AcknowledgeConfiguration(AcknowledgeConfigurationPacket),

    #[protocol_id(
        state = "configuration",
        bound = "clientbound",
        name = "minecraft:custom_payload"
    )]
    ConfigurationClientBoundPluginMessage(ConfigurationClientBoundPluginMessagePacket),

    #[protocol_id(
        state = "configuration",
        bound = "clientbound",
        name = "minecraft:select_known_packs"
    )]
    ClientBoundKnownPacks(ClientBoundKnownPacksPacket),

    #[protocol_id(
        state = "configuration",
        bound = "clientbound",
        name = "minecraft:registry_data"
    )]
    RegistryData(RegistryDataPacket),

    #[protocol_id(
        state = "configuration",
        bound = "clientbound",
        name = "minecraft:finish_configuration"
    )]
    FinishConfiguration(FinishConfigurationPacket),

    #[protocol_id(
        state = "configuration",
        bound = "clientbound",
        name = "minecraft:disconnect"
    )]
    ConfigurationDisconnect(DisconnectPacket),

    // Play packets
    #[protocol_id(state = "play", bound = "clientbound", name = "minecraft:login")]
    Login(Box<LoginPacket>),

    #[protocol_id(
        state = "play",
        bound = "clientbound",
        name = "minecraft:player_position"
    )]
    SynchronizePlayerPosition(SynchronizePlayerPositionPacket),

    #[protocol_id(
        state = "play",
        bound = "serverbound",
        name = "minecraft:move_player_pos"
    )]
    SetPlayerPosition(SetPlayerPositionPacket),

    #[protocol_id(
        state = "play",
        bound = "serverbound",
        name = "minecraft:move_player_pos_rot"
    )]
    SetPlayerPositionAndRotation(SetPlayerPositionAndRotationPacket),

    #[protocol_id(state = "play", bound = "serverbound", name = "minecraft:chat_command")]
    ChatCommand(ChatCommandPacket),

    #[protocol_id(state = "play", bound = "serverbound", name = "minecraft:chat")]
    ChatMessage(ChatMessagePacket),

    #[protocol_id(
        state = "play",
        bound = "clientbound",
        name = "minecraft:set_default_spawn_position"
    )]
    SetDefaultSpawnPosition(SetDefaultSpawnPositionPacket),

    #[protocol_id(state = "play", bound = "clientbound", name = "minecraft:commands")]
    Commands(CommandsPacket),

    #[protocol_id(state = "play", bound = "clientbound", name = "minecraft:game_event")]
    GameEvent(GameEventPacket),

    #[protocol_id(
        state = "play",
        bound = "clientbound",
        name = "minecraft:set_chunk_cache_center"
    )]
    SetCenterChunk(SetCenterChunkPacket),

    #[protocol_id(
        state = "play",
        bound = "clientbound",
        name = "minecraft:level_chunk_with_light"
    )]
    ChunkDataAndUpdateLight(Box<ChunkDataAndUpdateLightPacket>),

    #[protocol_id(
        state = "play",
        bound = "clientbound",
        name = "minecraft:custom_payload"
    )]
    PlayClientBoundPluginMessage(PlayClientBoundPluginMessagePacket),

    #[protocol_id(state = "play", bound = "clientbound", name = "minecraft:system_chat")]
    SystemChatMessage(SystemChatMessagePacket),

    #[protocol_id(
        state = "play",
        bound = "clientbound",
        name = "minecraft:legacy_chat_message"
    )]
    LegacyChatMessage(LegacyChatMessagePacket),

    #[protocol_id(state = "play", bound = "clientbound", name = "minecraft:keep_alive")]
    ClientBoundKeepAlive(ClientBoundKeepAlivePacket),

    #[protocol_id(state = "play", bound = "clientbound", name = "minecraft:disconnect")]
    PlayDisconnect(DisconnectPacket),

    #[protocol_id(state = "play", bound = "clientbound", name = "minecraft:set_time")]
    UpdateTime(UpdateTimePacket),

    #[protocol_id(state = "play", bound = "clientbound", name = "minecraft:tab_list")]
    TabList(TabListPacket),

    #[protocol_id(
        state = "play",
        bound = "clientbound",
        name = "minecraft:player_info_update"
    )]
    PlayerInfoUpdate(PlayerInfoUpdatePacket),

    #[protocol_id(
        state = "play",
        bound = "clientbound",
        name = "minecraft:set_entity_data"
    )]
    SetEntityMetadata(SetEntityMetadataPacket),

    #[protocol_id(state = "play", bound = "clientbound", name = "minecraft:boss_event")]
    BossBar(BossBarPacket),

    #[protocol_id(
        state = "play",
        bound = "clientbound",
        name = "minecraft:set_title_text"
    )]
    SetTitleText(SetTitleTextPacket),

    #[protocol_id(
        state = "play",
        bound = "clientbound",
        name = "minecraft:set_titles_animation"
    )]
    SetTitlesAnimation(SetTitlesAnimationPacket),

    #[protocol_id(
        state = "play",
        bound = "clientbound",
        name = "minecraft:set_subtitle_text"
    )]
    SetSubtitleText(SetTitleTextPacket),

    #[protocol_id(
        state = "play",
        bound = "clientbound",
        name = "minecraft:legacy_set_title"
    )]
    LegacySetTitle(LegacySetTitlePacket),

    #[protocol_id(
        state = "play",
        bound = "clientbound",
        name = "minecraft:set_action_bar_text"
    )]
    SetActionBarText(SetActionBarTextPacket),

    #[protocol_id(state = "play", bound = "clientbound", name = "minecraft:transfer")]
    Transfer(TransferPacket),

    #[protocol_id(
        state = "play",
        bound = "clientbound",
        name = "minecraft:player_abilities"
    )]
    ClientBoundPlayerAbilities(ClientBoundPlayerAbilitiesPacket),

    #[protocol_id(
        state = "play",
        bound = "serverbound",
        name = "minecraft:player_abilities"
    )]
    ServerBoundPlayerAbilities(ServerBoundPlayerAbilitiesPacket),

    #[protocol_id(
        state = "play",
        bound = "serverbound",
        name = "minecraft:pick_item_from_block"
    )]
    PickItemFromBlock(PickItemFromBlockPacket),
}

impl PacketHandler for PacketRegistry {
    fn handle(
        &self,
        client_state: &mut ClientState,
        server_state: &ServerState,
    ) -> Result<Batch<PacketRegistry>, PacketHandlerError> {
        match self {
            Self::Handshake(packet) => packet.handle(client_state, server_state),
            Self::StatusRequest(packet) => packet.handle(client_state, server_state),
            Self::PingRequest(packet) => packet.handle(client_state, server_state),
            Self::LoginStart(packet) => packet.handle(client_state, server_state),
            Self::CustomQueryAnswer(packet) => packet.handle(client_state, server_state),
            Self::LoginAcknowledged(packet) => packet.handle(client_state, server_state),
            Self::AcknowledgeConfiguration(packet) => packet.handle(client_state, server_state),
            Self::SetPlayerPositionAndRotation(packet) => packet.handle(client_state, server_state),
            Self::SetPlayerPosition(packet) => packet.handle(client_state, server_state),
            Self::ChatCommand(packet) => packet.handle(client_state, server_state),
            Self::ChatMessage(packet) => packet.handle(client_state, server_state),
            Self::ServerBoundPlayerAbilities(packet) => packet.handle(client_state, server_state),
            Self::PickItemFromBlock(packet) => packet.handle(client_state, server_state),
            _ => Err(PacketHandlerError::custom("Unhandled packet")),
        }
    }
}
