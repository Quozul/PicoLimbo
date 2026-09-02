use crate::play::data::death_location::DeathLocation;
use crate::play::data::global_pos::GlobalPos;
use minecraft_protocol::prelude::*;

#[derive(PacketOut)]
pub struct CommonPlayerSpawnInfo {
    #[protocol_version(min = V1_20_5)]
    pub v1_20_5_dimension_type: VarInt,
    #[protocol_version(max = V1_20_3)]
    pub dimension_type: Identifier,
    pub dimension_name: Identifier,
    pub hashed_seed: i64,
    #[protocol_version(max = V26_2)]
    pub game_mode: u8,
    #[protocol_version(max = V26_2)]
    pub previous_game_mode: i8,
    #[protocol_version(min = V26_3)]
    pub v26_3_game_mode: VarInt,
    #[protocol_version(min = V26_3)]
    pub v26_3_previous_game_mode: Optional<VarInt>,
    pub is_debug: bool,
    pub is_flat: bool,
    #[protocol_version(max = V26_2)]
    pub death_location: Optional<DeathLocation>,
    #[protocol_version(min = V26_3)]
    pub v26_3_death_location: Optional<GlobalPos>,
    pub portal_cooldown: VarInt,
    #[protocol_version(min = V1_21_2)]
    pub v1_21_2_sea_level: VarInt,
}

impl Default for CommonPlayerSpawnInfo {
    fn default() -> Self {
        let overworld = Identifier::vanilla_unchecked("overworld");
        Self {
            v1_20_5_dimension_type: VarInt::new(0),
            game_mode: 3,
            previous_game_mode: -1,
            v26_3_game_mode: VarInt::new(3),
            v26_3_previous_game_mode: Optional::None,
            is_debug: false,
            is_flat: true,
            death_location: Optional::None,
            v26_3_death_location: Optional::None,
            portal_cooldown: VarInt::default(),
            v1_21_2_sea_level: VarInt::new(63),
            dimension_name: overworld.clone(),
            dimension_type: overworld,
            hashed_seed: 0,
        }
    }
}

/// Min protocol version for this is 764 or 1.20.2 included
#[derive(PacketOut)]
pub struct PostV1_20_2Data {
    pub is_hardcore: bool,
    pub dimension_names: LengthPaddedVec<Identifier>,
    pub max_players: VarInt,
    pub view_distance: VarInt,
    pub simulation_distance: VarInt,
    pub reduced_debug_info: bool,
    pub enable_respawn_screen: bool,
    pub do_limited_crafting: bool,
    pub common_player_spawn_info: CommonPlayerSpawnInfo,
    #[protocol_version(min = V26_2)]
    pub v26_2_online_mode: bool,
    #[protocol_version(min = V1_20_5)]
    pub v1_20_5_enforces_secure_chat: bool,
}

impl Default for PostV1_20_2Data {
    fn default() -> Self {
        let overworld = Identifier::vanilla_unchecked("overworld");
        Self {
            is_hardcore: false,
            dimension_names: LengthPaddedVec::new(vec![overworld.clone()]),
            max_players: VarInt::new(1),
            view_distance: VarInt::new(10),
            simulation_distance: VarInt::new(10),
            reduced_debug_info: false,
            enable_respawn_screen: true,
            do_limited_crafting: false,
            v26_2_online_mode: false,
            v1_20_5_enforces_secure_chat: true,
            common_player_spawn_info: CommonPlayerSpawnInfo::default(),
        }
    }
}
