use crate::login::Property;
use minecraft_protocol::prelude::*;
use pico_text_component::prelude::Component;

#[derive(PacketOut)]
pub struct PlayerInfoUpdatePacket {
    #[protocol_version(max = V1_19_1)]
    action: VarInt,

    #[protocol_version(min = V1_19_3)]
    v1_19_3_mask: u8,
    players: LengthPaddedVec<Player>,
}

impl PlayerInfoUpdatePacket {
    pub fn skin(name: String, uuid: Uuid, property: Property, listed: bool) -> Self {
        Self::new(name, uuid, vec![property], listed)
    }

    pub fn skinless(name: String, uuid: Uuid, listed: bool) -> Self {
        Self::new(name, uuid, Vec::new(), listed)
    }

    fn new(name: String, uuid: Uuid, properties: Vec<Property>, listed: bool) -> Self {
        let properties = LengthPaddedVec::new(properties);
        let add_player_action = AddPlayer {
            name,
            properties,
            game_mode: VarInt::new(1),
            ping: VarInt::new(1),
            display_name: Optional::None,
            sig_data: Optional::None,
        };

        let actions = vec![
            PlayerActions::AddPlayer(add_player_action.clone()),
            PlayerActions::UpdateListed { listed },
        ];

        let mut mask = 0;
        for action in &actions {
            mask |= action.get_mask();
        }

        let player_action = Player {
            uuid: uuid.into(),
            action: add_player_action,
            actions,
        };

        Self {
            action: VarInt::new(0),
            v1_19_3_mask: mask,
            players: LengthPaddedVec::new(vec![player_action]),
        }
    }
}

#[derive(PacketOut)]
struct Player {
    uuid: UuidAsLongs,
    #[protocol_version(max = V1_19_1)]
    action: AddPlayer,
    #[protocol_version(min = V1_19_3)]
    actions: Vec<PlayerActions>,
}

#[derive(PacketOut, Clone)]
struct AddPlayer {
    name: String,
    properties: LengthPaddedVec<Property>,
    #[protocol_version(max = V1_19_1)]
    game_mode: VarInt,
    #[protocol_version(max = V1_19_1)]
    ping: VarInt,
    #[protocol_version(max = V1_19_1)]
    display_name: Optional<Component>,
    #[protocol_version(min = V1_19, max = V1_19_1)]
    sig_data: Optional<SigData>,
}

#[derive(PacketOut, Clone)]
struct SigData {
    timestamp: i64,
    public_key: LengthPaddedVec<i8>,
    signature: LengthPaddedVec<i8>,
}

#[derive(Clone)]
enum PlayerActions {
    AddPlayer(AddPlayer),
    UpdateListed { listed: bool },
}

impl PlayerActions {
    fn get_mask(&self) -> u8 {
        match self {
            PlayerActions::AddPlayer { .. } => 0x01,
            PlayerActions::UpdateListed { .. } => 0x08,
        }
    }
}

impl EncodePacket for PlayerActions {
    fn encode(
        &self,
        writer: &mut BinaryWriter,
        protocol_version: ProtocolVersion,
    ) -> Result<(), BinaryWriterError> {
        match self {
            PlayerActions::AddPlayer(value) => {
                value.encode(writer, protocol_version)?;
                Ok(())
            }
            PlayerActions::UpdateListed { listed } => {
                listed.encode(writer, protocol_version)?;
                Ok(())
            }
        }
    }
}
