use crate::play::data::death_location::DeathLocation;
use minecraft_protocol::prelude::*;

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
    #[pvn(766..)]
    pub v1_20_5_dimension_type: VarInt,
    #[pvn(..766)]
    pub dimension_type: Identifier,
    pub dimension_name: Identifier,
    pub hashed_seed: i64,
    pub game_mode: u8,
    pub previous_game_mode: i8,
    pub is_debug: bool,
    pub is_flat: bool,
    pub death_location: Optional<DeathLocation>,
    pub portal_cooldown: VarInt,
    #[pvn(768..)]
    pub v1_21_2_sea_level: VarInt,
    #[pvn(766..)]
    pub v1_20_5_enforces_secure_chat: bool,
}

impl Default for PostV1_20_2Data {
    fn default() -> Self {
        let overworld = Identifier::minecraft("overworld");
        Self {
            is_hardcore: false,
            dimension_names: LengthPaddedVec::new(vec![overworld.clone()]),
            max_players: VarInt::new(1),
            view_distance: VarInt::new(10),
            simulation_distance: VarInt::new(10),
            reduced_debug_info: false,
            enable_respawn_screen: true,
            do_limited_crafting: false,
            v1_20_5_dimension_type: VarInt::new(0),
            game_mode: 3,
            previous_game_mode: -1,
            is_debug: false,
            is_flat: true,
            death_location: Optional::None,
            portal_cooldown: VarInt::default(),
            v1_21_2_sea_level: VarInt::new(63),
            v1_20_5_enforces_secure_chat: true,
            dimension_name: overworld.clone(),
            dimension_type: overworld,
            hashed_seed: 0,
        }
    }
}
