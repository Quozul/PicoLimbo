use minecraft_protocol::prelude::*;
use thiserror::Error;

#[derive(Debug, PacketIn)]
#[packet_id("login/serverbound/minecraft:hello")]
pub struct LoginStartPacket {
    pub name: String,
    #[pvn(759..761)]
    sig_data: Option<SigData>,
    #[pvn(761..764)]
    v1_19_3_player_uuid: Option<Uuid>,
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

#[derive(Debug, Error)]
enum DecodeSigDataError {
    #[error("failed to decode number {0}")]
    DecodeNumberError(#[from] DecodeNumberError),
    #[error("failed to decode array {0}")]
    LengthPaddedVecDecodeError(#[from] LengthPaddedVecDecodeError<i8>),
}

#[derive(Debug, Default)]
#[allow(dead_code)]
struct SigData {
    /// When the key data will expire.
    timestamp: i64,
    /// Length of Public Key.
    public_key: LengthPaddedVec<i8>,
    /// The bytes of the public key signature the client received from Mojang.
    signature: LengthPaddedVec<i8>,
}

impl DecodePacketField for SigData {
    type Error = DecodeSigDataError;

    fn decode(bytes: &[u8], index: &mut usize) -> Result<Self, Self::Error> {
        let timestamp = i64::decode(bytes, index)?;
        let public_key = LengthPaddedVec::<i8>::decode(bytes, index)?;
        let signature = LengthPaddedVec::<i8>::decode(bytes, index)?;
        Ok(Self {
            timestamp,
            public_key,
            signature,
        })
    }
}
