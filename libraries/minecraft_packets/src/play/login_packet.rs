use data_types::prelude::{EncodePacketField, Identifier, LengthPaddedVec, VarInt};
use macros::PacketOut;
use protocol_version::ProtocolVersion::{V1_7_2, V1_21_4};

#[derive(Debug, PacketOut)]
pub struct LoginPacket {
    /// The player's Entity ID (EID).
    pub entity_id: i32,

    // 1.21.4 fields
    #[protocol(V1_21_4..)]
    pub is_hardcore: bool,
    /// Size of the following array.
    /// Identifiers for all dimensions on the server.
    #[protocol(V1_21_4..)]
    pub dimension_names: LengthPaddedVec<Identifier>,
    /// Was once used by the client to draw the player list, but now is ignored.
    #[protocol(V1_21_4..)]
    pub max_players: VarInt,
    /// Render distance (2-32).
    #[protocol(V1_21_4..)]
    pub view_distance: VarInt,
    /// The distance that the client will process specific things, such as entities.
    #[protocol(V1_21_4..)]
    pub simulation_distance: VarInt,
    /// If true, a Notchian client shows reduced information on the debug screen. For servers in development, this should almost always be false.
    #[protocol(V1_21_4..)]
    pub reduced_debug_info: bool,
    /// Set to false when the doImmediateRespawn gamerule is true.
    #[protocol(V1_21_4..)]
    pub enable_respawn_screen: bool,
    /// Whether players can only craft recipes they have already unlocked. Currently unused by the client.
    #[protocol(V1_21_4..)]
    pub do_limited_crafting: bool,
    /// The ID of the type of dimension in the minecraft:dimension_type registry, defined by the Registry Data packet.
    /// 0: overworld, 1: overworld_caves, 2: the_end, 3: the_nether
    #[protocol(V1_21_4..)]
    pub dimension_type: VarInt,
    /// Name of the dimension being spawned into.
    #[protocol(V1_21_4..)]
    pub dimension_name: Identifier,
    /// First 8 bytes of the SHA-256 hash of the world's seed. Used client side for biome noise
    #[protocol(V1_21_4..)]
    pub hashed_seed: i64,
    /// 0: Survival, 1: Creative, 2: Adventure, 3: Spectator.
    #[protocol(V1_21_4..)]
    pub game_mode: u8,
    /// -1: Undefined (null), 0: Survival, 1: Creative, 2: Adventure, 3: Spectator. The previous game mode. Vanilla client uses this for the debug (F3 + N & F3 + F4) game mode switch. (More information needed)
    #[protocol(V1_21_4..)]
    pub previous_game_mode: i8,
    /// True if the world is a debug mode world; debug mode worlds cannot be modified and have predefined blocks.
    #[protocol(V1_21_4..)]
    pub is_debug: bool,
    /// True if the world is a superflat world; flat worlds have different void fog and a horizon at y=0 instead of y=63
    #[protocol(V1_21_4..)]
    pub is_flat: bool,
    /// If true, then the next two fields are present.
    #[protocol(V1_21_4..)]
    pub has_death_location: bool,
    /// Name of the dimension the player died in.
    // pub death_dimension_name: Option<Identifier>,
    /// The location that the player died at.
    // pub death_location: Option<Position>,
    /// The number of ticks until the player can use the portal again.
    // pub portal_cooldown: VarInt,
    #[protocol(V1_21_4..)]
    pub unknown_a: VarInt,
    #[protocol(V1_21_4..)]
    pub unknown_b: VarInt,
    #[protocol(V1_21_4..)]
    pub enforces_secure_chat: bool,

    // 1.7.2 fields
    /// 0: Survival, 1: Creative, 2: Adventure, 3: Spectator. Bit 3 (0x8) is the hardcore flag.
    #[protocol(..=V1_7_2)]
    pub v1_7_2_gamemode_and_hardcore: u8,
    /// -1: Nether, 0: Overworld, 1: End
    #[protocol(..=V1_7_2)]
    pub v1_7_2_dimension: i8,
    /// 0: peaceful, 1: easy, 2: normal, 3: hard
    #[protocol(..=V1_7_2)]
    pub v1_7_2_difficulty: u8,
    #[protocol(..=V1_7_2)]
    pub v1_7_2_max_players: u8,
    /// default, flat, largeBiomes, amplified, default_1_1
    #[protocol(..=V1_7_2)]
    pub v1_7_2_level_type: String,
}

impl PacketId for LoginPacket {
    fn packet_id(protocol_version: &protocol_version::ProtocolVersion) -> Option<u8> {
        match protocol_version {
            V1_21_4 => Some(0x2C),
            s if s >= &V1_7_2 && s < &V1_21_4 => Some(0x23),
            _ => None,
        }
    }
}

impl Default for LoginPacket {
    fn default() -> Self {
        LoginPacket {
            entity_id: 0,
            is_hardcore: false,
            dimension_names: Vec::new().into(),
            max_players: VarInt::new(1),
            view_distance: VarInt::new(10),
            simulation_distance: VarInt::new(10),
            reduced_debug_info: false,
            enable_respawn_screen: true,
            do_limited_crafting: false,
            dimension_type: VarInt::new(0),
            dimension_name: Identifier::minecraft("overworld"),
            hashed_seed: 0,
            game_mode: 3,
            previous_game_mode: -1,
            is_debug: false,
            is_flat: false,
            has_death_location: false,
            // death_dimension_name: None,
            // death_location: None,
            // portal_cooldown: VarInt::default(),
            unknown_a: VarInt::default(),
            unknown_b: VarInt::default(),
            enforces_secure_chat: true,

            v1_7_2_gamemode_and_hardcore: 1,
            v1_7_2_dimension: 0,
            v1_7_2_difficulty: 0,
            v1_7_2_max_players: 1,
            v1_7_2_level_type: "flat".to_string(),
        }
    }
}
