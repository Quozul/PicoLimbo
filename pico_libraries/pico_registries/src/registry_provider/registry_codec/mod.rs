use crate::RegistryManager;
use protocol_version::protocol_version::ProtocolVersion;
use registry_codec_v1_16::get_registry_codec_bytes_v1_16;
use registry_codec_v1_16_2::get_registry_codec_bytes_v1_16_2;
use std::borrow::Cow;

mod registry_codec_v1_16;
mod registry_codec_v1_16_2;

pub fn get_registry_codec_v1_16(
    registry_manager: &RegistryManager,
    protocol_version: ProtocolVersion,
) -> crate::Result<Cow<'static, [u8]>> {
    crate::Error::incompatible_version(
        protocol_version,
        ProtocolVersion::V1_16,
        ProtocolVersion::V1_20_3,
    )?;
    if protocol_version.is_after_inclusive(ProtocolVersion::V1_16_2) {
        get_registry_codec_bytes_v1_16_2(registry_manager, protocol_version)
    } else {
        get_registry_codec_bytes_v1_16(registry_manager, protocol_version)
    }
}
