mod get_all_registries;
mod registries_indexes;

use crate::get_all_registries::{
    encode, get_v1_16_2_registry_codec, get_v1_16_registry_codec, get_v1_20_5_registries,
};
use crate::registries_indexes::{get_dimension_type_index, get_the_void_index};
use minecraft_protocol::prelude::{Dimension, Nbt, ProtocolVersion};
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use registries_data::registry_format::RegistryFormat;
use std::collections::HashSet;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::str::FromStr;
use std::{env, fs, path::PathBuf};

fn write_bytes_to_out_dir(out_dir: &Path, file_name: &str, data: &[u8]) -> anyhow::Result<()> {
    let path = out_dir.join(file_name);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut file = File::create(path)?;
    file.write_all(data)?;
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let data_location = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?)
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("data")
        .join("generated");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed={}", data_location.display());

    let out_dir = PathBuf::from(env::var("OUT_DIR")?);

    let mut canonical_versions: Vec<_> = ProtocolVersion::ALL_VERSION
        .iter()
        .map(|version| version.data())
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();
    canonical_versions.sort();

    let mut registries_arms = Vec::new();
    let mut dimensions_arms = Vec::new();
    let mut void_biome_arms = Vec::new();

    for &protocol_version in &canonical_versions {
        let registry_format = RegistryFormat::from_version(protocol_version);

        let version_string = protocol_version.to_string();
        let version_ident = Ident::new(&version_string, Span::call_site());

        let arm = match registry_format {
            RegistryFormat::V1_20_5 => {
                let registries = get_v1_20_5_registries(protocol_version, &data_location);
                let file_name = format!("registries_{}.bin", protocol_version);
                let bytes = encode(&registries, protocol_version);
                write_bytes_to_out_dir(&out_dir, &file_name, &bytes)?;

                let file_path_str = out_dir.join(file_name).to_str().unwrap().to_string();

                // TODO: Generate the code to instantiate V1_20_5Registries directly instead of decoding a binary
                // TODO: Share code between runtime and compiletime
                quote! {
                    ProtocolVersion::#version_ident => {
                        let bytes = include_bytes!(#file_path_str);
                        let mut reader = BinaryReader::new(bytes);
                        V1_20_5Registries::decode(&mut reader, protocol_version).map_or(Registries::None, |registries| {
                            Registries::V1_20_5 { registries }
                        })
                    },
                }
            }
            RegistryFormat::V1_20_2 => {
                let file_name = format!("registry_codec_{}.bin", protocol_version);
                let nbt = get_v1_16_2_registry_codec(protocol_version, &data_location);
                let bytes = encode(&nbt, protocol_version);
                write_bytes_to_out_dir(&out_dir, &file_name, &bytes)?;

                let file_path_str = out_dir.join(file_name).to_str().unwrap().to_string();

                quote! {
                    ProtocolVersion::#version_ident => Registries::V1_20_2 {
                        registry_codec: include_bytes!(#file_path_str)
                    },
                }
            }
            RegistryFormat::V1_19 => {
                let file_name = format!("registry_codec_{}.bin", protocol_version);
                let nbt = get_v1_16_2_registry_codec(protocol_version, &data_location);
                let bytes = encode(&nbt, protocol_version);
                write_bytes_to_out_dir(&out_dir, &file_name, &bytes)?;
                let file_path_str = out_dir.join(file_name).to_str().unwrap().to_string();

                quote! {
                    ProtocolVersion::#version_ident => Registries::V1_19 {
                        registry_codec: include_bytes!(#file_path_str)
                    },
                }
            }
            RegistryFormat::V1_16_2 => {
                let registry_codec = get_v1_16_2_registry_codec(protocol_version, &data_location);
                let registry_codec_bytes = encode(&registry_codec, protocol_version);
                let registry_codec_file_name = format!("registry_codec_{}.bin", protocol_version);
                write_bytes_to_out_dir(&out_dir, &registry_codec_file_name, &registry_codec_bytes)?;
                let registry_path_str = out_dir
                    .join(&registry_codec_file_name)
                    .to_str()
                    .unwrap()
                    .to_string();

                let dimension_types = registry_codec
                    .find_tag("minecraft:dimension_type")
                    .unwrap()
                    .find_tag("value")
                    .unwrap()
                    .get_nbt_vec()
                    .unwrap();

                let mut dimension_match_arms = Vec::<TokenStream>::new();

                for dimension_entry in dimension_types {
                    let name =
                        if let Some(Nbt::String { value, .. }) = dimension_entry.find_tag("name") {
                            value
                        } else {
                            continue;
                        };

                    match Dimension::from_str(name) {
                        Ok(dimension) => {
                            let element = if let Some(element) = dimension_entry.find_tag("element")
                            {
                                element
                            } else {
                                continue;
                            };

                            let dimension_bytes = encode(element, protocol_version);
                            let safe_name = name.replace(':', "_");
                            let dimension_file_name =
                                format!("dimension_{}_{}.bin", protocol_version, safe_name);
                            write_bytes_to_out_dir(
                                &out_dir,
                                &dimension_file_name,
                                &dimension_bytes,
                            )?;
                            let dimension_path_str = out_dir
                                .join(&dimension_file_name)
                                .to_str()
                                .unwrap()
                                .to_string();

                            let dimension_ident =
                                Ident::new(&dimension.to_string(), Span::call_site());

                            dimension_match_arms.push(
                                quote! { Dimension::#dimension_ident => include_bytes!(#dimension_path_str) as &'static [u8], },
                            );
                        }
                        Err(_) => {
                            continue;
                        }
                    }
                }

                quote! {
                    ProtocolVersion::#version_ident => {
                        let dimension_bytes = match dimension {
                            #(#dimension_match_arms)*
                        };
                        Registries::V1_16_2 {
                            registry_codec: include_bytes!(#registry_path_str),
                            dimension: dimension_bytes,
                        }
                    },
                }
            }
            RegistryFormat::V1_16 => {
                let file_name = format!("registry_codec_{}.bin", protocol_version);
                let nbt = get_v1_16_registry_codec(&data_location)?;
                let bytes = encode(&nbt, protocol_version);
                write_bytes_to_out_dir(&out_dir, &file_name, &bytes)?;
                let file_path_str = out_dir.join(file_name).to_str().unwrap().to_string();

                quote! {
                    ProtocolVersion::#version_ident => Registries::V1_16 {
                        registry_codec: include_bytes!(#file_path_str)
                    },
                }
            }
            RegistryFormat::None => quote! {},
        };
        if !arm.is_empty() {
            registries_arms.push(arm);
        }

        if protocol_version.is_after_inclusive(ProtocolVersion::V1_20_5) {
            let mut dimension_match_arms = Vec::<TokenStream>::new();

            for dimension in Dimension::ALL_DIMENSIONS {
                let dimension_ident = Ident::new(&dimension.to_string(), Span::call_site());
                let dimension_index = get_dimension_type_index(
                    protocol_version,
                    &data_location,
                    dimension.identifier().thing,
                );
                dimension_match_arms
                    .push(quote! { Dimension::#dimension_ident => Some(#dimension_index), });
            }

            let arm = quote! {
                ProtocolVersion::#version_ident => {
                    match dimension {
                        #(#dimension_match_arms)*
                    }
                },
            };
            dimensions_arms.push(arm);
        }

        if protocol_version.is_after_inclusive(ProtocolVersion::V1_19) {
            let void_biome_index = get_the_void_index(protocol_version, &data_location);
            let arm = quote! {
                ProtocolVersion::#version_ident => { Some(#void_biome_index) },
            };
            void_biome_arms.push(arm);
        }
    }

    let generated_code = quote! {
        #[allow(clippy::match_same_arms)]
        pub fn get_pregenerated_registries(protocol_version: minecraft_protocol::prelude::ProtocolVersion, dimension: minecraft_protocol::prelude::Dimension) -> Registries {
            match protocol_version {
                #(#registries_arms)*
                _ => Registries::None,
            }
        }

        #[allow(clippy::match_same_arms)]
        pub const fn get_pregenerated_dimension_index(protocol_version: minecraft_protocol::prelude::ProtocolVersion, dimension: minecraft_protocol::prelude::Dimension) -> Option<usize> {
               match protocol_version {
                #(#dimensions_arms)*
                _ => None,
            }
        }

        #[allow(clippy::match_same_arms)]
        pub const fn get_pregenerated_void_biome_index(protocol_version: minecraft_protocol::prelude::ProtocolVersion) -> Option<usize> {
               match protocol_version {
                #(#void_biome_arms)*
                _ => None,
            }
        }
    };

    let dest_path = out_dir.join("generated_registries.rs");
    fs::write(&dest_path, generated_code.to_string())?;

    Ok(())
}
