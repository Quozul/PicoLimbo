use minecraft_protocol::prelude::*;

/// This is the most important packet, good luck.
#[derive(PacketOut)]
pub struct LoginPacket {
    /// The player's Entity ID (EID).
    entity_id: i32,
    #[pvn(751..)]
    v1_16_2_is_hardcore: bool,
    #[pvn(..764)]
    game_mode: u8,
    #[pvn(735..764)]
    previous_game_mode: i8,
    /// Identifiers for all dimensions on the server.
    #[pvn(735..)]
    v1_16_dimension_names: LengthPaddedVec<Identifier>,
    /// Represents certain registries that are sent from the server and are applied on the client.
    #[pvn(735..764)]
    v1_16_registry_codec_bytes: Omitted<&'static [u8]>,
    /// The full extent of these is still unknown, but the tag represents a dimension and biome registry.
    #[pvn(751..759)]
    v1_16_2_dimension_codec_bytes: Omitted<&'static [u8]>,
    /// Name of the dimension type being spawned into.
    #[pvn(759..764)]
    v1_19_dimension_type: Identifier,
    /// Name of the dimension being spawned into.
    #[pvn(735..751)]
    v1_16_dimension_name: Identifier,
    #[pvn(735..764)]
    v1_16_world_name: Identifier,
    /// -1: Nether, 0: Overworld, 1: End; also, note that this is not a VarInt but instead a regular int.
    #[pvn(108..735)]
    v1_9_1_dimension: i32,
    #[pvn(..108)]
    /// -1: Nether, 0: Overworld, 1: End
    dimension: i8,
    /// First 8 bytes of the SHA-256 hash of the world's seed. Used client side for biome noise
    #[pvn(573..764)]
    hashed_seed: i64,
    /// Was once used by the client to draw the player list, but now is ignored.
    #[pvn(735..)]
    v1_16_max_players: VarInt,
    /// 0: peaceful, 1: easy, 2: normal, 3: hard
    #[pvn(..477)]
    difficulty: u8,
    #[pvn(..735)]
    max_players: u8,
    /// default, flat, largeBiomes, amplified, customized, buffet, default_1_1
    #[pvn(..735)]
    level_type: String,
    /// Render distance (2-32).
    #[pvn(477..)]
    view_distance: VarInt,
    /// The distance that the client will process specific things, such as entities.
    #[pvn(757..)]
    simulation_distance: VarInt,
    /// If true, a Notchian client shows reduced information on the debug screen. For servers in development, this should almost always be false.
    #[pvn(47..)]
    reduced_debug_info: bool,
    /// Set to false when the doImmediateRespawn gamerule is true.
    #[pvn(573..)]
    enable_respawn_screen: bool,
    /// Whether players can only craft recipes they have already unlocked. Currently unused by the client.
    #[pvn(764..)]
    v_1_20_2_do_limited_crafting: bool,
    /// The ID of the type of dimension in the minecraft:dimension_type registry, defined by the Registry Data packet.
    /// 0: overworld, 1: overworld_caves, 2: the_end, 3: the_nether
    #[pvn(766..)]
    v_1_20_5_dimension_type: VarInt,
    #[pvn(764..766)]
    v_1_20_2_dimension_type: Identifier,
    /// Name of the dimension being spawned into.
    #[pvn(764..)]
    v_1_20_2_dimension_name: Identifier,
    /// First 8 bytes of the SHA-256 hash of the world's seed. Used client side for biome noise
    #[pvn(764..)]
    v_1_20_2_hashed_seed: i64,
    /// 0: Survival, 1: Creative, 2: Adventure, 3: Spectator.
    #[pvn(764..)]
    v_1_20_2_game_mode: u8,
    /// -1: Undefined (null), 0: Survival, 1: Creative, 2: Adventure, 3: Spectator. The previous game mode. Vanilla client uses this for the debug (F3 + N & F3 + F4) game mode switch. (More information needed)
    #[pvn(764..)]
    v_1_20_2_previous_game_mode: i8,
    /// True if the world is a debug mode world; debug mode worlds cannot be modified and have predefined blocks.
    #[pvn(735..)]
    is_debug: bool,
    /// True if the world is a superflat world; flat worlds have different void fog and a horizon at y=0 instead of y=63.
    #[pvn(735..)]
    is_flat: bool,
    #[pvn(759..)]
    has_death_location: Optional<DeathLocation>,
    /// The number of ticks until the player can use the portal again.
    #[pvn(763..)]
    portal_cooldown: VarInt,
    #[pvn(768..)]
    sea_level: VarInt,
    #[pvn(766..)]
    enforces_secure_chat: bool,
}

#[derive(PacketOut)]
struct DeathLocation {
    /// Name of the dimension the player died in.
    death_dimension_name: Omitted<Identifier>,
    /// The location that the player died at.
    death_location: Omitted<Position>,
}

impl Default for LoginPacket {
    fn default() -> Self {
        let overworld = Identifier::minecraft("overworld");
        Self {
            entity_id: 0,
            v1_16_2_is_hardcore: false,
            game_mode: 3,
            previous_game_mode: -1,
            v1_16_dimension_names: LengthPaddedVec::new(vec![overworld.clone()]),
            v1_16_registry_codec_bytes: Omitted::None,
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
            v1_16_2_dimension_codec_bytes: Omitted::None,
            v1_16_world_name: overworld.clone(),
            v1_9_1_dimension: 0,
            dimension: 0,
            hashed_seed: 0,
            v_1_20_2_game_mode: 3,
            v_1_20_2_previous_game_mode: -1,
            is_debug: false,
            is_flat: true,
            has_death_location: Optional::None,
            portal_cooldown: VarInt::default(),
            sea_level: VarInt::new(63),
            enforces_secure_chat: false,
            v_1_20_2_dimension_name: overworld.clone(),
            v_1_20_2_dimension_type: overworld,
            v_1_20_2_hashed_seed: 0,
            difficulty: 0,
        }
    }
}

impl LoginPacket {
    /// This is the constructor for version 1.16.2 up to 1.18.2 included
    pub fn with_dimension_codec(
        dimension: Dimension,
        registry_codec_bytes: &'static [u8],
        dimension_codec_bytes: &'static [u8],
    ) -> Self {
        let iden = dimension.identifier();
        Self {
            // dimension names (1.16+)
            v1_16_dimension_names: LengthPaddedVec::new(vec![iden.clone()]),
            v1_16_world_name: iden.clone(),

            // dimension type identifiers (1.19+, 1.20.2)
            v1_19_dimension_type: iden.clone(),

            // leave absolutely everything else as the default
            v1_16_registry_codec_bytes: Omitted::Some(registry_codec_bytes),
            v1_16_2_dimension_codec_bytes: Omitted::Some(dimension_codec_bytes),
            ..Self::default()
        }
    }

    /// This is the constructor for 1.16, 1.16.1 and 1.19 up to 1.20 included
    pub fn with_registry_codec(dimension: Dimension, registry_codec_bytes: &'static [u8]) -> Self {
        let iden = dimension.identifier();
        Self {
            // dimension names (1.16+)
            v1_16_dimension_names: LengthPaddedVec::new(vec![iden.clone()]),
            v1_16_world_name: iden.clone(),
            v1_16_dimension_name: iden.clone(),

            // dimension type identifiers (1.19+)
            v1_19_dimension_type: iden.clone(),

            // leave absolutely everything else as the default
            v1_16_registry_codec_bytes: Omitted::Some(registry_codec_bytes),
            ..Self::default()
        }
    }

    /// This is the constructor for all versions from 1.20.2 to 1.20.4 included and versions from 1.7.2 to 1.15.2
    pub fn with_dimension(dimension: Dimension) -> Self {
        let iden = dimension.identifier();
        Self {
            // legacy fields
            dimension: dimension.legacy_i8(),
            v1_9_1_dimension: dimension.legacy_i32(),

            // dimension names (1.16+)
            v1_16_dimension_names: LengthPaddedVec::new(vec![iden.clone()]),

            // dimension type and name
            v_1_20_2_dimension_name: iden.clone(),
            v_1_20_2_dimension_type: iden.clone(),
            ..Self::default()
        }
    }

    /// This is the constructor for all versions starting 1.20.5
    pub fn with_dimension_index(dimension: Dimension, dimension_index: i32) -> Self {
        let iden = dimension.identifier();
        Self {
            // dimension names (1.16+)
            v1_16_dimension_names: LengthPaddedVec::new(vec![iden.clone()]),

            // dimension type and name
            v_1_20_2_dimension_name: iden.clone(),

            // dimension type index (1.20.5)
            v_1_20_5_dimension_type: dimension_index.into(),
            ..Self::default()
        }
    }

    pub fn set_game_mode(mut self, game_mode: u8) -> Self {
        if self.v1_16_2_is_hardcore {
            self.game_mode = game_mode | 0x8;
        } else {
            self.game_mode = game_mode;
        }
        self.v_1_20_2_game_mode = game_mode;
        self
    }

    pub fn set_view_distance(mut self, view_distance: i32) -> Self {
        self.view_distance = VarInt::new(view_distance);
        self.simulation_distance = VarInt::new(view_distance);
        self
    }

    pub fn set_hardcore(mut self, protocol_version: ProtocolVersion, hardcore: bool) -> Self {
        self.v1_16_2_is_hardcore = hardcore;
        if hardcore && protocol_version.is_before_inclusive(ProtocolVersion::V1_16_1) {
            self.game_mode |= 0x8;
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn expected_snapshots() -> HashMap<i32, Vec<u8>> {
        HashMap::from([
            (
                769,
                vec![
                    0, 0, 0, 0, 0, 1, 19, 109, 105, 110, 101, 99, 114, 97, 102, 116, 58, 111, 118,
                    101, 114, 119, 111, 114, 108, 100, 1, 10, 10, 0, 1, 0, 0, 19, 109, 105, 110,
                    101, 99, 114, 97, 102, 116, 58, 111, 118, 101, 114, 119, 111, 114, 108, 100, 0,
                    0, 0, 0, 0, 0, 0, 0, 3, 255, 0, 1, 0, 0, 63, 0,
                ],
            ),
            (
                768,
                vec![
                    0, 0, 0, 0, 0, 1, 19, 109, 105, 110, 101, 99, 114, 97, 102, 116, 58, 111, 118,
                    101, 114, 119, 111, 114, 108, 100, 1, 10, 10, 0, 1, 0, 0, 19, 109, 105, 110,
                    101, 99, 114, 97, 102, 116, 58, 111, 118, 101, 114, 119, 111, 114, 108, 100, 0,
                    0, 0, 0, 0, 0, 0, 0, 3, 255, 0, 1, 0, 0, 63, 0,
                ],
            ),
            (
                767,
                vec![
                    0, 0, 0, 0, 0, 1, 19, 109, 105, 110, 101, 99, 114, 97, 102, 116, 58, 111, 118,
                    101, 114, 119, 111, 114, 108, 100, 1, 10, 10, 0, 1, 0, 0, 19, 109, 105, 110,
                    101, 99, 114, 97, 102, 116, 58, 111, 118, 101, 114, 119, 111, 114, 108, 100, 0,
                    0, 0, 0, 0, 0, 0, 0, 3, 255, 0, 1, 0, 0, 0,
                ],
            ),
            (
                766,
                vec![
                    0, 0, 0, 0, 0, 1, 19, 109, 105, 110, 101, 99, 114, 97, 102, 116, 58, 111, 118,
                    101, 114, 119, 111, 114, 108, 100, 1, 10, 10, 0, 1, 0, 0, 19, 109, 105, 110,
                    101, 99, 114, 97, 102, 116, 58, 111, 118, 101, 114, 119, 111, 114, 108, 100, 0,
                    0, 0, 0, 0, 0, 0, 0, 3, 255, 0, 1, 0, 0, 0,
                ],
            ),
            (
                765,
                vec![
                    0, 0, 0, 0, 0, 1, 19, 109, 105, 110, 101, 99, 114, 97, 102, 116, 58, 111, 118,
                    101, 114, 119, 111, 114, 108, 100, 1, 10, 10, 0, 1, 0, 19, 109, 105, 110, 101,
                    99, 114, 97, 102, 116, 58, 111, 118, 101, 114, 119, 111, 114, 108, 100, 19,
                    109, 105, 110, 101, 99, 114, 97, 102, 116, 58, 111, 118, 101, 114, 119, 111,
                    114, 108, 100, 0, 0, 0, 0, 0, 0, 0, 0, 3, 255, 0, 1, 0, 0,
                ],
            ),
            (
                764,
                vec![
                    0, 0, 0, 0, 0, 1, 19, 109, 105, 110, 101, 99, 114, 97, 102, 116, 58, 111, 118,
                    101, 114, 119, 111, 114, 108, 100, 1, 10, 10, 0, 1, 0, 19, 109, 105, 110, 101,
                    99, 114, 97, 102, 116, 58, 111, 118, 101, 114, 119, 111, 114, 108, 100, 19,
                    109, 105, 110, 101, 99, 114, 97, 102, 116, 58, 111, 118, 101, 114, 119, 111,
                    114, 108, 100, 0, 0, 0, 0, 0, 0, 0, 0, 3, 255, 0, 1, 0, 0,
                ],
            ),
            (
                763,
                vec![
                    0, 0, 0, 0, 0, 3, 255, 1, 19, 109, 105, 110, 101, 99, 114, 97, 102, 116, 58,
                    111, 118, 101, 114, 119, 111, 114, 108, 100, 8, 0, 5, 72, 101, 108, 108, 111,
                    0, 5, 87, 111, 114, 108, 100, 19, 109, 105, 110, 101, 99, 114, 97, 102, 116,
                    58, 111, 118, 101, 114, 119, 111, 114, 108, 100, 19, 109, 105, 110, 101, 99,
                    114, 97, 102, 116, 58, 111, 118, 101, 114, 119, 111, 114, 108, 100, 0, 0, 0, 0,
                    0, 0, 0, 0, 1, 10, 10, 0, 1, 0, 1, 0, 0,
                ],
            ),
            (
                762,
                vec![
                    0, 0, 0, 0, 0, 3, 255, 1, 19, 109, 105, 110, 101, 99, 114, 97, 102, 116, 58,
                    111, 118, 101, 114, 119, 111, 114, 108, 100, 8, 0, 5, 72, 101, 108, 108, 111,
                    0, 5, 87, 111, 114, 108, 100, 19, 109, 105, 110, 101, 99, 114, 97, 102, 116,
                    58, 111, 118, 101, 114, 119, 111, 114, 108, 100, 19, 109, 105, 110, 101, 99,
                    114, 97, 102, 116, 58, 111, 118, 101, 114, 119, 111, 114, 108, 100, 0, 0, 0, 0,
                    0, 0, 0, 0, 1, 10, 10, 0, 1, 0, 1, 0,
                ],
            ),
            (
                761,
                vec![
                    0, 0, 0, 0, 0, 3, 255, 1, 19, 109, 105, 110, 101, 99, 114, 97, 102, 116, 58,
                    111, 118, 101, 114, 119, 111, 114, 108, 100, 8, 0, 5, 72, 101, 108, 108, 111,
                    0, 5, 87, 111, 114, 108, 100, 19, 109, 105, 110, 101, 99, 114, 97, 102, 116,
                    58, 111, 118, 101, 114, 119, 111, 114, 108, 100, 19, 109, 105, 110, 101, 99,
                    114, 97, 102, 116, 58, 111, 118, 101, 114, 119, 111, 114, 108, 100, 0, 0, 0, 0,
                    0, 0, 0, 0, 1, 10, 10, 0, 1, 0, 1, 0,
                ],
            ),
            (
                760,
                vec![
                    0, 0, 0, 0, 0, 3, 255, 1, 19, 109, 105, 110, 101, 99, 114, 97, 102, 116, 58,
                    111, 118, 101, 114, 119, 111, 114, 108, 100, 8, 0, 5, 72, 101, 108, 108, 111,
                    0, 5, 87, 111, 114, 108, 100, 19, 109, 105, 110, 101, 99, 114, 97, 102, 116,
                    58, 111, 118, 101, 114, 119, 111, 114, 108, 100, 19, 109, 105, 110, 101, 99,
                    114, 97, 102, 116, 58, 111, 118, 101, 114, 119, 111, 114, 108, 100, 0, 0, 0, 0,
                    0, 0, 0, 0, 1, 10, 10, 0, 1, 0, 1, 0,
                ],
            ),
            (
                759,
                vec![
                    0, 0, 0, 0, 0, 3, 255, 1, 19, 109, 105, 110, 101, 99, 114, 97, 102, 116, 58,
                    111, 118, 101, 114, 119, 111, 114, 108, 100, 8, 0, 5, 72, 101, 108, 108, 111,
                    0, 5, 87, 111, 114, 108, 100, 19, 109, 105, 110, 101, 99, 114, 97, 102, 116,
                    58, 111, 118, 101, 114, 119, 111, 114, 108, 100, 19, 109, 105, 110, 101, 99,
                    114, 97, 102, 116, 58, 111, 118, 101, 114, 119, 111, 114, 108, 100, 0, 0, 0, 0,
                    0, 0, 0, 0, 1, 10, 10, 0, 1, 0, 1, 0,
                ],
            ),
            (
                758,
                vec![
                    0, 0, 0, 0, 0, 3, 255, 1, 19, 109, 105, 110, 101, 99, 114, 97, 102, 116, 58,
                    111, 118, 101, 114, 119, 111, 114, 108, 100, 8, 0, 5, 72, 101, 108, 108, 111,
                    0, 5, 87, 111, 114, 108, 100, 8, 0, 5, 72, 101, 108, 108, 111, 0, 5, 87, 111,
                    114, 108, 100, 19, 109, 105, 110, 101, 99, 114, 97, 102, 116, 58, 111, 118,
                    101, 114, 119, 111, 114, 108, 100, 0, 0, 0, 0, 0, 0, 0, 0, 1, 10, 10, 0, 1, 0,
                    1,
                ],
            ),
            (
                757,
                vec![
                    0, 0, 0, 0, 0, 3, 255, 1, 19, 109, 105, 110, 101, 99, 114, 97, 102, 116, 58,
                    111, 118, 101, 114, 119, 111, 114, 108, 100, 8, 0, 5, 72, 101, 108, 108, 111,
                    0, 5, 87, 111, 114, 108, 100, 8, 0, 5, 72, 101, 108, 108, 111, 0, 5, 87, 111,
                    114, 108, 100, 19, 109, 105, 110, 101, 99, 114, 97, 102, 116, 58, 111, 118,
                    101, 114, 119, 111, 114, 108, 100, 0, 0, 0, 0, 0, 0, 0, 0, 1, 10, 10, 0, 1, 0,
                    1,
                ],
            ),
            (
                756,
                vec![
                    0, 0, 0, 0, 0, 3, 255, 1, 19, 109, 105, 110, 101, 99, 114, 97, 102, 116, 58,
                    111, 118, 101, 114, 119, 111, 114, 108, 100, 8, 0, 5, 72, 101, 108, 108, 111,
                    0, 5, 87, 111, 114, 108, 100, 8, 0, 5, 72, 101, 108, 108, 111, 0, 5, 87, 111,
                    114, 108, 100, 19, 109, 105, 110, 101, 99, 114, 97, 102, 116, 58, 111, 118,
                    101, 114, 119, 111, 114, 108, 100, 0, 0, 0, 0, 0, 0, 0, 0, 1, 10, 0, 1, 0, 1,
                ],
            ),
            (
                755,
                vec![
                    0, 0, 0, 0, 0, 3, 255, 1, 19, 109, 105, 110, 101, 99, 114, 97, 102, 116, 58,
                    111, 118, 101, 114, 119, 111, 114, 108, 100, 8, 0, 5, 72, 101, 108, 108, 111,
                    0, 5, 87, 111, 114, 108, 100, 8, 0, 5, 72, 101, 108, 108, 111, 0, 5, 87, 111,
                    114, 108, 100, 19, 109, 105, 110, 101, 99, 114, 97, 102, 116, 58, 111, 118,
                    101, 114, 119, 111, 114, 108, 100, 0, 0, 0, 0, 0, 0, 0, 0, 1, 10, 0, 1, 0, 1,
                ],
            ),
            (
                754,
                vec![
                    0, 0, 0, 0, 0, 3, 255, 1, 19, 109, 105, 110, 101, 99, 114, 97, 102, 116, 58,
                    111, 118, 101, 114, 119, 111, 114, 108, 100, 8, 0, 5, 72, 101, 108, 108, 111,
                    0, 5, 87, 111, 114, 108, 100, 8, 0, 5, 72, 101, 108, 108, 111, 0, 5, 87, 111,
                    114, 108, 100, 19, 109, 105, 110, 101, 99, 114, 97, 102, 116, 58, 111, 118,
                    101, 114, 119, 111, 114, 108, 100, 0, 0, 0, 0, 0, 0, 0, 0, 1, 10, 0, 1, 0, 1,
                ],
            ),
            (
                753,
                vec![
                    0, 0, 0, 0, 0, 3, 255, 1, 19, 109, 105, 110, 101, 99, 114, 97, 102, 116, 58,
                    111, 118, 101, 114, 119, 111, 114, 108, 100, 8, 0, 5, 72, 101, 108, 108, 111,
                    0, 5, 87, 111, 114, 108, 100, 8, 0, 5, 72, 101, 108, 108, 111, 0, 5, 87, 111,
                    114, 108, 100, 19, 109, 105, 110, 101, 99, 114, 97, 102, 116, 58, 111, 118,
                    101, 114, 119, 111, 114, 108, 100, 0, 0, 0, 0, 0, 0, 0, 0, 1, 10, 0, 1, 0, 1,
                ],
            ),
            (
                751,
                vec![
                    0, 0, 0, 0, 0, 3, 255, 1, 19, 109, 105, 110, 101, 99, 114, 97, 102, 116, 58,
                    111, 118, 101, 114, 119, 111, 114, 108, 100, 8, 0, 5, 72, 101, 108, 108, 111,
                    0, 5, 87, 111, 114, 108, 100, 8, 0, 5, 72, 101, 108, 108, 111, 0, 5, 87, 111,
                    114, 108, 100, 19, 109, 105, 110, 101, 99, 114, 97, 102, 116, 58, 111, 118,
                    101, 114, 119, 111, 114, 108, 100, 0, 0, 0, 0, 0, 0, 0, 0, 1, 10, 0, 1, 0, 1,
                ],
            ),
            (
                736,
                vec![
                    0, 0, 0, 0, 3, 255, 1, 19, 109, 105, 110, 101, 99, 114, 97, 102, 116, 58, 111,
                    118, 101, 114, 119, 111, 114, 108, 100, 8, 0, 5, 72, 101, 108, 108, 111, 0, 5,
                    87, 111, 114, 108, 100, 19, 109, 105, 110, 101, 99, 114, 97, 102, 116, 58, 111,
                    118, 101, 114, 119, 111, 114, 108, 100, 19, 109, 105, 110, 101, 99, 114, 97,
                    102, 116, 58, 111, 118, 101, 114, 119, 111, 114, 108, 100, 0, 0, 0, 0, 0, 0, 0,
                    0, 1, 10, 0, 1, 0, 1,
                ],
            ),
            (
                735,
                vec![
                    0, 0, 0, 0, 3, 255, 1, 19, 109, 105, 110, 101, 99, 114, 97, 102, 116, 58, 111,
                    118, 101, 114, 119, 111, 114, 108, 100, 8, 0, 5, 72, 101, 108, 108, 111, 0, 5,
                    87, 111, 114, 108, 100, 19, 109, 105, 110, 101, 99, 114, 97, 102, 116, 58, 111,
                    118, 101, 114, 119, 111, 114, 108, 100, 19, 109, 105, 110, 101, 99, 114, 97,
                    102, 116, 58, 111, 118, 101, 114, 119, 111, 114, 108, 100, 0, 0, 0, 0, 0, 0, 0,
                    0, 1, 10, 0, 1, 0, 1,
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

    static NBT_BYTES: &[u8] = &[
        8, 0, 5, 72, 101, 108, 108, 111, 0, 5, 87, 111, 114, 108, 100,
    ];

    fn create_packet() -> LoginPacket {
        LoginPacket {
            v1_16_registry_codec_bytes: Omitted::Some(NBT_BYTES),
            v1_16_2_dimension_codec_bytes: Omitted::Some(NBT_BYTES),
            ..LoginPacket::default()
        }
    }

    #[test]
    fn login_packet() {
        let snapshots = expected_snapshots();
        let packet = create_packet();

        for (version, expected_bytes) in snapshots {
            let mut writer = BinaryWriter::new();
            packet
                .encode(&mut writer, ProtocolVersion::from(version))
                .unwrap();
            let bytes = writer.into_inner();
            assert_eq!(bytes, expected_bytes, "Mismatch for version {}", version);
        }
    }
}
