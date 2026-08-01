use minecraft_protocol::prelude::*;

#[derive(Default, PacketIn)]
pub struct LoginStartPacket {
    pub name: String,
    #[protocol_version(min = V1_19, max = V1_19_1)]
    #[allow(dead_code)]
    sig_data: Optional<SigData>,
    #[protocol_version(min = V1_19_3, max = V1_20)]
    v1_19_3_player_uuid: Optional<Uuid>, // Really??
    #[protocol_version(min = V1_20_2)]
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
