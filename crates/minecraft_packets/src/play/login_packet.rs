use crate::play::data::dimension::Dimension;
use minecraft_protocol::prelude::*;

/// This is the most important packet, good luck.
#[derive(PacketOut)]
#[packet_id("play/clientbound/minecraft:login")]
pub struct LoginPacket {
    /// The player's Entity ID (EID).
    pub entity_id: i32,
    #[pvn(751..)]
    pub is_hardcore: bool,
    #[pvn(..764)]
    pub game_mode: u8,
    #[pvn(735..764)]
    pub previous_game_mode: i8,
    /// Identifiers for all dimensions on the server.
    #[pvn(735..)]
    pub v1_16_dimension_names: LengthPaddedVec<Identifier>,
    /// Represents certain registries that are sent from the server and are applied on the client.
    #[pvn(735..764)]
    pub registry_codec: Nbt,
    /// The full extent of these is still unknown, but the tag represents a dimension and biome registry.
    #[pvn(751..759)]
    pub v1_16_dimension_codec: Nbt,
    /// Name of the dimension type being spawned into.
    #[pvn(759..764)]
    pub v1_19_dimension_type: Identifier,
    /// Name of the dimension being spawned into.
    #[pvn(735..751)]
    pub v1_16_dimension_name: Identifier,
    #[pvn(735..764)]
    pub v1_16_world_name: Identifier,
    /// -1: Nether, 0: Overworld, 1: End; also, note that this is not a VarInt but instead a regular int.
    #[pvn(108..735)]
    pub v1_9_1_dimension: i32,
    #[pvn(..108)]
    /// -1: Nether, 0: Overworld, 1: End
    pub dimension: i8,
    /// First 8 bytes of the SHA-256 hash of the world's seed. Used client side for biome noise
    #[pvn(573..764)]
    pub hashed_seed: i64,
    /// Was once used by the client to draw the player list, but now is ignored.
    #[pvn(735..)]
    pub v1_16_max_players: VarInt,
    /// 0: peaceful, 1: easy, 2: normal, 3: hard
    #[pvn(..477)]
    pub difficulty: u8,
    #[pvn(..735)]
    pub max_players: u8,
    /// default, flat, largeBiomes, amplified, customized, buffet, default_1_1
    #[pvn(..735)]
    pub level_type: String,
    /// Render distance (2-32).
    #[pvn(477..)]
    pub view_distance: VarInt,
    /// The distance that the client will process specific things, such as entities.
    #[pvn(757..)]
    pub simulation_distance: VarInt,
    /// If true, a Notchian client shows reduced information on the debug screen. For servers in development, this should almost always be false.
    #[pvn(47..)]
    pub reduced_debug_info: bool,
    /// Set to false when the doImmediateRespawn gamerule is true.
    #[pvn(573..)]
    pub enable_respawn_screen: bool,
    /// Whether players can only craft recipes they have already unlocked. Currently unused by the client.
    #[pvn(764..)]
    pub v_1_20_2_do_limited_crafting: bool,
    /// The ID of the type of dimension in the minecraft:dimension_type registry, defined by the Registry Data packet.
    /// 0: overworld, 1: overworld_caves, 2: the_end, 3: the_nether
    #[pvn(766..)]
    pub v_1_20_5_dimension_type: VarInt,
    #[pvn(764..766)]
    pub v_1_20_2_dimension_type: Identifier,
    /// Name of the dimension being spawned into.
    #[pvn(764..)]
    pub v_1_20_2_dimension_name: Identifier,
    /// First 8 bytes of the SHA-256 hash of the world's seed. Used client side for biome noise
    #[pvn(764..)]
    pub v_1_20_2_hashed_seed: i64,
    /// 0: Survival, 1: Creative, 2: Adventure, 3: Spectator.
    #[pvn(764..)]
    pub v_1_20_2_game_mode: u8,
    /// -1: Undefined (null), 0: Survival, 1: Creative, 2: Adventure, 3: Spectator. The previous game mode. Vanilla client uses this for the debug (F3 + N & F3 + F4) game mode switch. (More information needed)
    #[pvn(764..)]
    pub v_1_20_2_previous_game_mode: i8,
    /// True if the world is a debug mode world; debug mode worlds cannot be modified and have predefined blocks.
    #[pvn(735..)]
    pub is_debug: bool,
    /// True if the world is a superflat world; flat worlds have different void fog and a horizon at y=0 instead of y=63.
    #[pvn(735..)]
    pub is_flat: bool,
    /// If true, then the next two fields are present.
    #[pvn(759..)]
    pub has_death_location: bool,
    /// Name of the dimension the player died in.
    #[pvn(759..)]
    pub death_dimension_name: Option<Identifier>,
    /// The location that the player died at.
    #[pvn(759..)]
    pub death_location: Option<Position>,
    /// The number of ticks until the player can use the portal again.
    #[pvn(763..)]
    pub portal_cooldown: VarInt,
    #[pvn(768..)]
    pub sea_level: VarInt,
    #[pvn(766..)]
    pub enforces_secure_chat: bool,
}

impl Default for LoginPacket {
    fn default() -> Self {
        let overworld = Identifier::minecraft("overworld");
        Self {
            entity_id: 0,
            is_hardcore: false,
            game_mode: 3,
            previous_game_mode: -1,
            v1_16_dimension_names: Vec::new().into(),
            registry_codec: Nbt::End,
            v1_16_max_players: VarInt::new(1),
            max_players: 1,
            level_type: "default".to_string(),
            view_distance: VarInt::new(10),
            simulation_distance: VarInt::new(10),
            reduced_debug_info: false,
            enable_respawn_screen: true,
            v1_16_dimension_name: overworld.clone(),
            v_1_20_2_do_limited_crafting: false,
            v_1_20_5_dimension_type: VarInt::new(0),
            v1_19_dimension_type: overworld.clone(),
            v1_16_dimension_codec: Nbt::End,
            v1_16_world_name: overworld.clone(),
            v1_9_1_dimension: 0,
            dimension: 0,
            hashed_seed: 0,
            v_1_20_2_game_mode: 3,
            v_1_20_2_previous_game_mode: -1,
            is_debug: false,
            is_flat: false,
            has_death_location: false,
            death_dimension_name: None,
            death_location: None,
            portal_cooldown: VarInt::default(),
            sea_level: VarInt::default(),
            enforces_secure_chat: false,
            v_1_20_2_dimension_name: overworld.clone(),
            v_1_20_2_dimension_type: overworld,
            v_1_20_2_hashed_seed: 0,
            difficulty: 0,
        }
    }
}

impl LoginPacket {
    pub fn new_with_codecs(
        dimension: &Dimension,
        registry_codec: Nbt,
        dimension_codec: Nbt,
    ) -> Self {
        let iden = dimension.identifier();
        Self {
            // legacy fields
            dimension: dimension.legacy_i8(),
            v1_9_1_dimension: dimension.legacy_i32(),

            // dimension names (1.16+)
            v1_16_world_name: iden.clone(),
            v1_16_dimension_name: iden.clone(),
            v_1_20_2_dimension_name: iden.clone(),

            // dimension type identifiers (1.19+, 1.20.2)
            v1_19_dimension_type: iden.clone(),
            v_1_20_2_dimension_type: iden.clone(),

            // dimension type index (1.20.5)
            v_1_20_5_dimension_type: dimension.type_index_1_20_5(),

            // leave absolutely everything else as the default
            registry_codec,
            v1_16_dimension_codec: dimension_codec,
            ..Self::default()
        }
    }

    pub fn new_with_dimension(dimension: &Dimension) -> Self {
        Self::new_with_codecs(dimension, Nbt::End, Nbt::End)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn expected_snapshots() -> HashMap<u32, Vec<u8>> {
        HashMap::from([
            (
                769,
                vec![
                    0, 0, 0, 0, 0, 0, 1, 10, 10, 0, 1, 0, 0, 19, 109, 105, 110, 101, 99, 114, 97,
                    102, 116, 58, 111, 118, 101, 114, 119, 111, 114, 108, 100, 0, 0, 0, 0, 0, 0, 0,
                    0, 3, 255, 0, 0, 0, 0, 0, 0,
                ],
            ),
            (
                768,
                vec![
                    0, 0, 0, 0, 0, 0, 1, 10, 10, 0, 1, 0, 0, 19, 109, 105, 110, 101, 99, 114, 97,
                    102, 116, 58, 111, 118, 101, 114, 119, 111, 114, 108, 100, 0, 0, 0, 0, 0, 0, 0,
                    0, 3, 255, 0, 0, 0, 0, 0, 0,
                ],
            ),
            (
                767,
                vec![
                    0, 0, 0, 0, 0, 0, 1, 10, 10, 0, 1, 0, 0, 19, 109, 105, 110, 101, 99, 114, 97,
                    102, 116, 58, 111, 118, 101, 114, 119, 111, 114, 108, 100, 0, 0, 0, 0, 0, 0, 0,
                    0, 3, 255, 0, 0, 0, 0, 0,
                ],
            ),
            (
                766,
                vec![
                    0, 0, 0, 0, 0, 0, 1, 10, 10, 0, 1, 0, 0, 19, 109, 105, 110, 101, 99, 114, 97,
                    102, 116, 58, 111, 118, 101, 114, 119, 111, 114, 108, 100, 0, 0, 0, 0, 0, 0, 0,
                    0, 3, 255, 0, 0, 0, 0, 0,
                ],
            ),
            (
                765,
                vec![
                    0, 0, 0, 0, 0, 0, 1, 10, 10, 0, 1, 0, 19, 109, 105, 110, 101, 99, 114, 97, 102,
                    116, 58, 111, 118, 101, 114, 119, 111, 114, 108, 100, 19, 109, 105, 110, 101,
                    99, 114, 97, 102, 116, 58, 111, 118, 101, 114, 119, 111, 114, 108, 100, 0, 0,
                    0, 0, 0, 0, 0, 0, 3, 255, 0, 0, 0, 0,
                ],
            ),
            (
                764,
                vec![
                    0, 0, 0, 0, 0, 0, 1, 10, 10, 0, 1, 0, 19, 109, 105, 110, 101, 99, 114, 97, 102,
                    116, 58, 111, 118, 101, 114, 119, 111, 114, 108, 100, 19, 109, 105, 110, 101,
                    99, 114, 97, 102, 116, 58, 111, 118, 101, 114, 119, 111, 114, 108, 100, 0, 0,
                    0, 0, 0, 0, 0, 0, 3, 255, 0, 0, 0, 0,
                ],
            ),
            (
                763,
                vec![
                    0, 0, 0, 0, 0, 3, 255, 0, 8, 0, 5, 72, 101, 108, 108, 111, 0, 5, 87, 111, 114,
                    108, 100, 19, 109, 105, 110, 101, 99, 114, 97, 102, 116, 58, 111, 118, 101,
                    114, 119, 111, 114, 108, 100, 19, 109, 105, 110, 101, 99, 114, 97, 102, 116,
                    58, 111, 118, 101, 114, 119, 111, 114, 108, 100, 0, 0, 0, 0, 0, 0, 0, 0, 1, 10,
                    10, 0, 1, 0, 0, 0, 0,
                ],
            ),
            (
                762,
                vec![
                    0, 0, 0, 0, 0, 3, 255, 0, 8, 0, 5, 72, 101, 108, 108, 111, 0, 5, 87, 111, 114,
                    108, 100, 19, 109, 105, 110, 101, 99, 114, 97, 102, 116, 58, 111, 118, 101,
                    114, 119, 111, 114, 108, 100, 19, 109, 105, 110, 101, 99, 114, 97, 102, 116,
                    58, 111, 118, 101, 114, 119, 111, 114, 108, 100, 0, 0, 0, 0, 0, 0, 0, 0, 1, 10,
                    10, 0, 1, 0, 0, 0,
                ],
            ),
            (
                761,
                vec![
                    0, 0, 0, 0, 0, 3, 255, 0, 8, 0, 5, 72, 101, 108, 108, 111, 0, 5, 87, 111, 114,
                    108, 100, 19, 109, 105, 110, 101, 99, 114, 97, 102, 116, 58, 111, 118, 101,
                    114, 119, 111, 114, 108, 100, 19, 109, 105, 110, 101, 99, 114, 97, 102, 116,
                    58, 111, 118, 101, 114, 119, 111, 114, 108, 100, 0, 0, 0, 0, 0, 0, 0, 0, 1, 10,
                    10, 0, 1, 0, 0, 0,
                ],
            ),
            (
                760,
                vec![
                    0, 0, 0, 0, 0, 3, 255, 0, 8, 0, 5, 72, 101, 108, 108, 111, 0, 5, 87, 111, 114,
                    108, 100, 19, 109, 105, 110, 101, 99, 114, 97, 102, 116, 58, 111, 118, 101,
                    114, 119, 111, 114, 108, 100, 19, 109, 105, 110, 101, 99, 114, 97, 102, 116,
                    58, 111, 118, 101, 114, 119, 111, 114, 108, 100, 0, 0, 0, 0, 0, 0, 0, 0, 1, 10,
                    10, 0, 1, 0, 0, 0,
                ],
            ),
            (
                759,
                vec![
                    0, 0, 0, 0, 0, 3, 255, 0, 8, 0, 5, 72, 101, 108, 108, 111, 0, 5, 87, 111, 114,
                    108, 100, 19, 109, 105, 110, 101, 99, 114, 97, 102, 116, 58, 111, 118, 101,
                    114, 119, 111, 114, 108, 100, 19, 109, 105, 110, 101, 99, 114, 97, 102, 116,
                    58, 111, 118, 101, 114, 119, 111, 114, 108, 100, 0, 0, 0, 0, 0, 0, 0, 0, 1, 10,
                    10, 0, 1, 0, 0, 0,
                ],
            ),
            (
                758,
                vec![
                    0, 0, 0, 0, 0, 3, 255, 0, 8, 0, 5, 72, 101, 108, 108, 111, 0, 5, 87, 111, 114,
                    108, 100, 8, 0, 5, 72, 101, 108, 108, 111, 0, 5, 87, 111, 114, 108, 100, 19,
                    109, 105, 110, 101, 99, 114, 97, 102, 116, 58, 111, 118, 101, 114, 119, 111,
                    114, 108, 100, 0, 0, 0, 0, 0, 0, 0, 0, 1, 10, 10, 0, 1, 0, 0,
                ],
            ),
            (
                757,
                vec![
                    0, 0, 0, 0, 0, 3, 255, 0, 8, 0, 5, 72, 101, 108, 108, 111, 0, 5, 87, 111, 114,
                    108, 100, 8, 0, 5, 72, 101, 108, 108, 111, 0, 5, 87, 111, 114, 108, 100, 19,
                    109, 105, 110, 101, 99, 114, 97, 102, 116, 58, 111, 118, 101, 114, 119, 111,
                    114, 108, 100, 0, 0, 0, 0, 0, 0, 0, 0, 1, 10, 10, 0, 1, 0, 0,
                ],
            ),
            (
                756,
                vec![
                    0, 0, 0, 0, 0, 3, 255, 0, 8, 0, 5, 72, 101, 108, 108, 111, 0, 5, 87, 111, 114,
                    108, 100, 8, 0, 5, 72, 101, 108, 108, 111, 0, 5, 87, 111, 114, 108, 100, 19,
                    109, 105, 110, 101, 99, 114, 97, 102, 116, 58, 111, 118, 101, 114, 119, 111,
                    114, 108, 100, 0, 0, 0, 0, 0, 0, 0, 0, 1, 10, 0, 1, 0, 0,
                ],
            ),
            (
                755,
                vec![
                    0, 0, 0, 0, 0, 3, 255, 0, 8, 0, 5, 72, 101, 108, 108, 111, 0, 5, 87, 111, 114,
                    108, 100, 8, 0, 5, 72, 101, 108, 108, 111, 0, 5, 87, 111, 114, 108, 100, 19,
                    109, 105, 110, 101, 99, 114, 97, 102, 116, 58, 111, 118, 101, 114, 119, 111,
                    114, 108, 100, 0, 0, 0, 0, 0, 0, 0, 0, 1, 10, 0, 1, 0, 0,
                ],
            ),
            (
                754,
                vec![
                    0, 0, 0, 0, 0, 3, 255, 0, 8, 0, 5, 72, 101, 108, 108, 111, 0, 5, 87, 111, 114,
                    108, 100, 8, 0, 5, 72, 101, 108, 108, 111, 0, 5, 87, 111, 114, 108, 100, 19,
                    109, 105, 110, 101, 99, 114, 97, 102, 116, 58, 111, 118, 101, 114, 119, 111,
                    114, 108, 100, 0, 0, 0, 0, 0, 0, 0, 0, 1, 10, 0, 1, 0, 0,
                ],
            ),
            (
                753,
                vec![
                    0, 0, 0, 0, 0, 3, 255, 0, 8, 0, 5, 72, 101, 108, 108, 111, 0, 5, 87, 111, 114,
                    108, 100, 8, 0, 5, 72, 101, 108, 108, 111, 0, 5, 87, 111, 114, 108, 100, 19,
                    109, 105, 110, 101, 99, 114, 97, 102, 116, 58, 111, 118, 101, 114, 119, 111,
                    114, 108, 100, 0, 0, 0, 0, 0, 0, 0, 0, 1, 10, 0, 1, 0, 0,
                ],
            ),
            (
                751,
                vec![
                    0, 0, 0, 0, 0, 3, 255, 0, 8, 0, 5, 72, 101, 108, 108, 111, 0, 5, 87, 111, 114,
                    108, 100, 8, 0, 5, 72, 101, 108, 108, 111, 0, 5, 87, 111, 114, 108, 100, 19,
                    109, 105, 110, 101, 99, 114, 97, 102, 116, 58, 111, 118, 101, 114, 119, 111,
                    114, 108, 100, 0, 0, 0, 0, 0, 0, 0, 0, 1, 10, 0, 1, 0, 0,
                ],
            ),
            (
                736,
                vec![
                    0, 0, 0, 0, 3, 255, 0, 8, 0, 5, 72, 101, 108, 108, 111, 0, 5, 87, 111, 114,
                    108, 100, 19, 109, 105, 110, 101, 99, 114, 97, 102, 116, 58, 111, 118, 101,
                    114, 119, 111, 114, 108, 100, 19, 109, 105, 110, 101, 99, 114, 97, 102, 116,
                    58, 111, 118, 101, 114, 119, 111, 114, 108, 100, 0, 0, 0, 0, 0, 0, 0, 0, 1, 10,
                    0, 1, 0, 0,
                ],
            ),
            (
                735,
                vec![
                    0, 0, 0, 0, 3, 255, 0, 8, 0, 5, 72, 101, 108, 108, 111, 0, 5, 87, 111, 114,
                    108, 100, 19, 109, 105, 110, 101, 99, 114, 97, 102, 116, 58, 111, 118, 101,
                    114, 119, 111, 114, 108, 100, 19, 109, 105, 110, 101, 99, 114, 97, 102, 116,
                    58, 111, 118, 101, 114, 119, 111, 114, 108, 100, 0, 0, 0, 0, 0, 0, 0, 0, 1, 10,
                    0, 1, 0, 0,
                ],
            ),
            (
                578,
                vec![
                    0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 7, 100, 101, 102, 97,
                    117, 108, 116, 10, 0, 1,
                ],
            ),
            (
                575,
                vec![
                    0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 7, 100, 101, 102, 97,
                    117, 108, 116, 10, 0, 1,
                ],
            ),
            (
                573,
                vec![
                    0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 7, 100, 101, 102, 97,
                    117, 108, 116, 10, 0, 1,
                ],
            ),
            (
                498,
                vec![
                    0, 0, 0, 0, 3, 0, 0, 0, 0, 1, 7, 100, 101, 102, 97, 117, 108, 116, 10, 0,
                ],
            ),
            (
                490,
                vec![
                    0, 0, 0, 0, 3, 0, 0, 0, 0, 1, 7, 100, 101, 102, 97, 117, 108, 116, 10, 0,
                ],
            ),
            (
                485,
                vec![
                    0, 0, 0, 0, 3, 0, 0, 0, 0, 1, 7, 100, 101, 102, 97, 117, 108, 116, 10, 0,
                ],
            ),
            (
                480,
                vec![
                    0, 0, 0, 0, 3, 0, 0, 0, 0, 1, 7, 100, 101, 102, 97, 117, 108, 116, 10, 0,
                ],
            ),
            (
                477,
                vec![
                    0, 0, 0, 0, 3, 0, 0, 0, 0, 1, 7, 100, 101, 102, 97, 117, 108, 116, 10, 0,
                ],
            ),
            (
                404,
                vec![
                    0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 1, 7, 100, 101, 102, 97, 117, 108, 116, 0,
                ],
            ),
            (
                401,
                vec![
                    0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 1, 7, 100, 101, 102, 97, 117, 108, 116, 0,
                ],
            ),
            (
                393,
                vec![
                    0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 1, 7, 100, 101, 102, 97, 117, 108, 116, 0,
                ],
            ),
            (
                340,
                vec![
                    0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 1, 7, 100, 101, 102, 97, 117, 108, 116, 0,
                ],
            ),
            (
                338,
                vec![
                    0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 1, 7, 100, 101, 102, 97, 117, 108, 116, 0,
                ],
            ),
            (
                335,
                vec![
                    0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 1, 7, 100, 101, 102, 97, 117, 108, 116, 0,
                ],
            ),
            (
                316,
                vec![
                    0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 1, 7, 100, 101, 102, 97, 117, 108, 116, 0,
                ],
            ),
            (
                315,
                vec![
                    0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 1, 7, 100, 101, 102, 97, 117, 108, 116, 0,
                ],
            ),
            (
                210,
                vec![
                    0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 1, 7, 100, 101, 102, 97, 117, 108, 116, 0,
                ],
            ),
            (
                110,
                vec![
                    0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 1, 7, 100, 101, 102, 97, 117, 108, 116, 0,
                ],
            ),
            (
                109,
                vec![
                    0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 1, 7, 100, 101, 102, 97, 117, 108, 116, 0,
                ],
            ),
            (
                108,
                vec![
                    0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 1, 7, 100, 101, 102, 97, 117, 108, 116, 0,
                ],
            ),
            (
                107,
                vec![
                    0, 0, 0, 0, 3, 0, 0, 1, 7, 100, 101, 102, 97, 117, 108, 116, 0,
                ],
            ),
            (
                47,
                vec![
                    0, 0, 0, 0, 3, 0, 0, 1, 7, 100, 101, 102, 97, 117, 108, 116, 0,
                ],
            ),
            (
                5,
                vec![0, 0, 0, 0, 3, 0, 0, 1, 7, 100, 101, 102, 97, 117, 108, 116],
            ),
            (
                4,
                vec![0, 0, 0, 0, 3, 0, 0, 1, 7, 100, 101, 102, 97, 117, 108, 116],
            ),
        ])
    }

    fn create_packet() -> LoginPacket {
        LoginPacket {
            registry_codec: Nbt::String {
                name: Some("Hello".to_string()),
                value: "World".to_string(),
            },
            v1_16_dimension_codec: Nbt::String {
                name: Some("Hello".to_string()),
                value: "World".to_string(),
            },
            ..LoginPacket::default()
        }
    }

    #[test]
    fn login_packet() {
        let snapshots = expected_snapshots();
        let packet = create_packet();

        for (version, expected_bytes) in snapshots {
            let bytes = packet.encode(version).unwrap();
            assert_eq!(bytes, expected_bytes, "Mismatch for version {}", version);
        }
    }
}
