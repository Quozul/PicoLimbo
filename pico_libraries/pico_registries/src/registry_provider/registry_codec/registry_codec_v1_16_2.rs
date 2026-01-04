use crate::RegistryManager;
use crate::registry_provider::shared::{encode_nameless_compound_to_bytes, get_registry_keys};
use pico_nbt2::{IndexMap, Value};
use protocol_version::protocol_version::ProtocolVersion;
use serde::Serialize;
use std::borrow::Cow;
use std::num::TryFromIntError;

#[derive(Serialize)]
struct RegistryCodec {
    #[serde(rename = "type")]
    registry_type: String,
    value: Vec<RegistryCodecEntry>,
}

#[derive(Serialize)]
struct RegistryCodecEntry {
    name: String,
    id: i32,
    element: Value,
}

pub fn get_registry_codec_bytes_v1_16_2(
    registry_manager: &RegistryManager,
    protocol_version: ProtocolVersion,
) -> crate::Result<Cow<'static, [u8]>> {
    crate::Error::incompatible_version(
        protocol_version,
        ProtocolVersion::V1_16_2,
        ProtocolVersion::V1_20_3,
    )?;
    let registries = get_registry_keys(protocol_version)?;

    let registries = registries
        .iter()
        .filter_map(|registry_keys| registry_manager.try_get(registry_keys));

    let mut final_registries = IndexMap::new();
    for registry in registries {
        let registry_type = registry.get_registry_key().get_value().to_string();
        final_registries.insert(
            registry_type.clone(),
            RegistryCodec {
                registry_type,
                value: registry
                    .get_entries()
                    .iter()
                    .enumerate()
                    .flat_map(
                        |(index, entry)| -> Result<RegistryCodecEntry, TryFromIntError> {
                            Ok(RegistryCodecEntry {
                                name: entry.get_registry_key().get_value().to_string(),
                                id: i32::try_from(index)?,
                                element: entry.get_raw_value().clone(),
                            })
                        },
                    )
                    .collect(),
            },
        );
    }

    Ok(encode_nameless_compound_to_bytes(
        protocol_version,
        &final_registries,
    )?)
}
