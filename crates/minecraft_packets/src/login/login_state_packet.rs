use minecraft_protocol::prelude::*;

#[derive(Default, PacketIn)]
pub struct LoginStartPacket {
    pub name: String,
    #[pvn(759..761)]
    #[allow(dead_code)]
    sig_data: Optional<SigData>,
    #[pvn(761..764)]
    v1_19_3_player_uuid: Optional<Uuid>, // Really??
    #[pvn(764..)]
    v1_20_2_player_uuid: Uuid,
}

impl LoginStartPacket {
    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn uuid(&self) -> Uuid {
        self.v1_19_3_player_uuid.unwrap_or(self.v1_20_2_player_uuid)
    }
}

#[derive(Default, PacketIn)]
#[allow(dead_code)]
struct SigData {
    /// When the key data will expire.
    timestamp: i64,
    /// Length of Public Key.
    public_key: LengthPaddedVec<i8>,
    /// The bytes of the public key signature the client received from Mojang.
    signature: LengthPaddedVec<i8>,
}
