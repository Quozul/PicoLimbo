use crate::registry_provider::shared::encode_nameless_compound_to_bytes;
use crate::{RegistryKeys, RegistryManager};
use pico_nbt2::Value;
use protocol_version::protocol_version::ProtocolVersion;
use serde::Serialize;
use std::borrow::Cow;

#[derive(Serialize)]
struct RegistryCodecEntry {
    name: String,
    #[serde(flatten)]
    element: Value,
}

#[derive(Serialize)]
struct RegistryCodec {
    // We can actually have more than only dimensions, but this is the only mandatory registry
    dimension: Vec<RegistryCodecEntry>,
}

pub fn get_registry_codec_bytes_v1_16(
    registry_manager: &RegistryManager,
    protocol_version: ProtocolVersion,
) -> crate::Result<Cow<'static, [u8]>> {
    crate::Error::incompatible_version(
        protocol_version,
        ProtocolVersion::V1_16,
        ProtocolVersion::V1_16_1,
    )?;
    let registry = registry_manager.get(&RegistryKeys::DimensionType)?;

    let root = RegistryCodec {
        dimension: registry
            .get_entries()
            .iter()
            .map(|entry| RegistryCodecEntry {
                name: entry.get_registry_key().get_value().to_string(),
                element: entry.get_raw_value().clone(),
            })
            .collect(),
    };

    Ok(encode_nameless_compound_to_bytes(protocol_version, &root)?)
}
