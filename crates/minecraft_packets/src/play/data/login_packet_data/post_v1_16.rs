use crate::play::data::death_location::DeathLocation;
use minecraft_protocol::prelude::*;
use std::borrow::Cow;

/// Min protocol version for this is 735 or 1.16 included
/// Max protocol version for this is 763 or 1.20 included
#[derive(PacketOut)]
pub struct PostV1_16Data {
    #[pvn(751..)]
    pub v1_16_2_is_hardcore: bool,
    pub game_mode: u8,
    pub previous_game_mode: i8,
    pub dimension_names: LengthPaddedVec<Identifier>,
    pub registry_codec_bytes: Omitted<Cow<'static, [u8]>>,
    #[pvn(751..759)]
    pub v1_16_2_dimension_codec_bytes: Omitted<Cow<'static, [u8]>>,
    #[pvn(759..)]
    pub v1_19_dimension_type: Identifier,
    #[pvn(..751)]
    pub dimension_name: Identifier,
    pub world_name: Identifier,
    pub hashed_seed: i64,
    pub max_players: VarInt,
    pub view_distance: VarInt,
    #[pvn(757..)]
    pub v1_18_simulation_distance: VarInt,
    pub reduced_debug_info: bool,
    pub enable_respawn_screen: bool,
    pub is_debug: bool,
    pub is_flat: bool,
    #[pvn(759..)]
    pub v1_19_has_death_location: Optional<DeathLocation>,
    #[pvn(763..)]
    pub v1_20_portal_cooldown: VarInt,
}

impl Default for PostV1_16Data {
    fn default() -> Self {
        let overworld = Identifier::vanilla_unchecked("overworld");
        Self {
            v1_16_2_is_hardcore: false,
            game_mode: 3,
            previous_game_mode: -1,
            dimension_names: LengthPaddedVec::new(vec![overworld.clone()]),
            registry_codec_bytes: Omitted::None,
            max_players: VarInt::new(1),
            view_distance: VarInt::new(10),
            v1_18_simulation_distance: VarInt::new(10),
            reduced_debug_info: false,
            enable_respawn_screen: true,
            dimension_name: overworld.clone(),
            v1_19_dimension_type: overworld.clone(),
            v1_16_2_dimension_codec_bytes: Omitted::None,
            world_name: overworld.clone(),
            hashed_seed: 0,
            is_debug: false,
            is_flat: true,
            v1_19_has_death_location: Optional::None,
            v1_20_portal_cooldown: VarInt::default(),
        }
    }
}
