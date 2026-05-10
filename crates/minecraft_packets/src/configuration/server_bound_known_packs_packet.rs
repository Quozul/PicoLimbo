use crate::configuration::data::known_pack::KnownPack;
use minecraft_protocol::prelude::*;

/// Sent by the client in response to `ClientBoundKnownPacksPacket`. Lists the
/// known packs the client accepts (i.e. that the client has locally). The
/// server can use this to decide whether to send full registry data or rely on
/// the client's bundled vanilla data.
#[derive(PacketIn)]
pub struct ServerBoundKnownPacksPacket {
    pub known_packs: LengthPaddedVec<KnownPack>,
}
