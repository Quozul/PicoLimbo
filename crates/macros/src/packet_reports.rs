use proc_macro::TokenStream;
use protocol_version::protocol_version::ProtocolVersion;
use quote::{format_ident, quote};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;
use syn::parse::{Parse, ParseStream};
use syn::{Data, DeriveInput, Error, Fields, Ident, LitStr, Token, parse_macro_input};

/// Represents the "protocol_id" object within the JSON.
#[derive(Debug, Deserialize)]
struct PacketInfo {
    protocol_id: u8,
}

/// Represents the mapping from packet_name to PacketInfo.
type DirectionPackets = HashMap<String, PacketInfo>;

/// Represents the mapping from direction (serverbound/clientbound) to DirectionPackets.
type StateDirections = HashMap<String, DirectionPackets>;

/// Represents the top-level structure of the JSON: state -> directions -> packets.
#[derive(Debug, Deserialize)]
struct RawPacketData(HashMap<String, StateDirections>);

struct PacketVariantInfo {
    variant_ident: Ident,
    packet_type: syn::Path,
    state: String,
    bound: String,
    name: String,
}

pub fn packet_report_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let enum_ident = &input.ident;

    let variants = parse_enum_variants(&input.data);
    let protocol_data = load_all_protocol_data();

    let decode_impl = generate_decode_impl(&variants, &protocol_data);
    let encode_impl = generate_encode_impl(enum_ident, &variants, &protocol_data);

    let expanded = quote! {
        #[derive(Debug, thiserror::Error)]
        pub enum PacketRegistryEncodeError {
            #[error("Encode error: Version {0} does not support packet {1}")]
            UnsupportedPacket(ProtocolVersion, String),
            #[error("Encode error: This packet cannot be encoded")]
            CannotBeEncoded,
            #[error("Failed to write packet")]
            Encode(#[from] BinaryWriterError),
        }

        #[derive(Debug, thiserror::Error)]
        pub enum PacketRegistryDecodeError {
            #[error("Decode error: Packet id is missing from the payload")]
            MissingPacketId,
            #[error("Decode error: Unknown version version={1} packet_id={0}")]
            UnknownVersion(u8, i32),
            #[error("Decode error: Packet not found version={0} state={1} packet_id={2}")]
            NoCorrespondingPacket(i32, State, u8),
            #[error("Failed to read packet")]
            Decode(#[from] BinaryReaderError),
        }

        impl #enum_ident {
            #decode_impl
            #encode_impl
        }
    };

    TokenStream::from(expanded)
}

fn parse_enum_variants(data: &Data) -> Vec<PacketVariantInfo> {
    let variants = match data {
        Data::Enum(data_enum) => &data_enum.variants,
        _ => panic!("PacketReport can only be derived for enums"),
    };

    variants
        .iter()
        .map(|variant| {
            let fields = match &variant.fields {
                Fields::Unnamed(fields) if fields.unnamed.len() == 1 => &fields.unnamed,
                _ => panic!("Enum variants must have exactly one unnamed field (the packet type)"),
            };
            let packet_type = match &fields.first().unwrap().ty {
                syn::Type::Path(type_path) => type_path.path.clone(),
                _ => panic!("Expected a path type for the packet struct"),
            };

            let attr = variant
                .attrs
                .iter()
                .find(|a| a.path().is_ident("protocol_id"))
                .expect("Each variant must have a #[protocol_id] attribute");

            let Ok(ProtocolIdAttribute { state, bound, name }) =
                attr.parse_args::<ProtocolIdAttribute>()
            else {
                panic!("Failed to parse #[protocol_id] attribute")
            };

            PacketVariantInfo {
                variant_ident: variant.ident.clone(),
                packet_type,
                state: state.expect("state missing").value(),
                bound: bound.expect("bound missing").value(),
                name: name.expect("name missing").value(),
            }
        })
        .collect()
}

/// Parses the `#[protocol_id(state = "...", bound = "...", name = "...")]` attribute.
pub struct ProtocolIdAttribute {
    pub state: Option<LitStr>,
    pub bound: Option<LitStr>,
    pub name: Option<LitStr>,
}

impl Parse for ProtocolIdAttribute {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut state: Option<LitStr> = None;
        let mut bound: Option<LitStr> = None;
        let mut name: Option<LitStr> = None;

        if input.is_empty() {
            panic!("Packet metadata missing")
        }

        let mut parse_kv = |input: ParseStream| -> syn::Result<()> {
            let ident: Ident = input.parse()?;
            input.parse::<Token![=]>()?;
            let value: LitStr = input.parse()?;

            if ident == "state" {
                if state.is_some() {
                    return Err(Error::new(ident.span(), "duplicate `state` field"));
                }
                state = Some(value);
            } else if ident == "bound" {
                if bound.is_some() {
                    return Err(Error::new(ident.span(), "duplicate `bound` field"));
                }
                bound = Some(value);
            } else if ident == "name" {
                if name.is_some() {
                    return Err(Error::new(ident.span(), "duplicate `name` field"));
                }
                name = Some(value);
            } else {
                return Err(Error::new(
                    ident.span(),
                    "expected either `state`, `bound` or `name`",
                ));
            }
            Ok(())
        };

        parse_kv(input)?;
        while input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
            parse_kv(input)?;
        }

        Ok(Self { state, bound, name })
    }
}

fn load_all_protocol_data() -> HashMap<String, RawPacketData> {
    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let data_dir = manifest_dir
        .parent()
        .unwrap()
        .join("data")
        .join("generated");

    let mut all_data = HashMap::new();

    for entry in fs::read_dir(data_dir).expect("Failed to read data/generated directory") {
        let entry = entry.unwrap();
        if entry.file_type().unwrap().is_dir() {
            let version_name = entry.file_name().into_string().unwrap();
            let report_path = entry.path().join("reports").join("packets.json");

            if report_path.exists() {
                let content = fs::read_to_string(&report_path)
                    .unwrap_or_else(|_| panic!("Failed to read {:?}", report_path));
                let report: RawPacketData = serde_json::from_str(&content)
                    .unwrap_or_else(|e| panic!("Failed to parse {:?}: {}", report_path, e));
                all_data.insert(version_name, report);
            }
        }
    }
    all_data
}

fn generate_decode_impl(
    variants: &[PacketVariantInfo],
    protocol_data: &HashMap<String, RawPacketData>,
) -> proc_macro2::TokenStream {
    let report_arms = protocol_data.iter().map(|(version_name, report)| {
        let version_number = ProtocolVersion::from_str(version_name).expect("Failed to parse version name").version_number();
        let state_arms = variants
            .iter()
            .filter(|v| v.bound == "serverbound")
            .filter_map(|variant_info| {
                let state_str = &variant_info.state;
                let packet_name = &variant_info.name;

                let packet_info = report.0.get(state_str)
                    .and_then(|directions| directions.get("serverbound"))
                    .and_then(|packets| packets.get(packet_name));

                if let Some(packet_info) = packet_info {
                    let id = packet_info.protocol_id;
                    let state_ident = format_ident!("{}", capitalize_first(state_str));
                    let packet_type = &variant_info.packet_type;
                    let variant_ident = &variant_info.variant_ident;

                    Some(quote! {
                        (State::#state_ident, #id) => {
                            let packet = <#packet_type as DecodePacket>::decode(&mut payload, protocol_version)?;
                            return Ok(Self::#variant_ident(packet));
                        }
                    })
                } else {
                    None
                }
            });

        quote! {
            #version_number => {
                match (state, packet_id) {
                    #(#state_arms)*
                    _ => return Err(PacketRegistryDecodeError::NoCorrespondingPacket(packets_version, state, packet_id)),
                }
            }
        }
    });

    quote! {
        pub fn decode_packet(
            protocol_version: ProtocolVersion,
            state: State,
            raw_packet: RawPacket,
        ) -> Result<Self, PacketRegistryDecodeError> {
            match raw_packet.packet_id() {
                Some(packet_id) => {
                    let packets_version = protocol_version.packets().version_number();
                    let mut payload = BinaryReader::new(raw_packet.data());
                    match packets_version {
                        #(#report_arms)*
                        _ => return Err(PacketRegistryDecodeError::UnknownVersion(packet_id, packets_version)),
                    }
                }
                None => {
                    Err(PacketRegistryDecodeError::MissingPacketId)
                }
            }
        }
    }
}

fn generate_encode_impl(
    enum_ident: &Ident,
    variants: &[PacketVariantInfo],
    protocol_data: &HashMap<String, RawPacketData>,
) -> proc_macro2::TokenStream {
    let variant_arms = variants
        .iter()
        .filter(|v| v.bound == "clientbound")
        .map(|variant_info| {
            let variant_ident = &variant_info.variant_ident;
            let packet_name = &variant_info.name;
            let state_str = &variant_info.state;

            let report_arms = protocol_data.iter().filter_map(|(version_name, report)| {
                let version_number = ProtocolVersion::from_str(version_name)
                    .expect("Failed to parse version name")
                    .version_number();
                let packet_info = report
                    .0
                    .get(state_str)
                    .and_then(|directions| directions.get("clientbound"))
                    .and_then(|packets| packets.get(packet_name));

                if let Some(packet_info) = packet_info {
                    let id = packet_info.protocol_id;
                    Some(quote! { #version_number => #id, })
                } else {
                    None
                }
            });

            quote! {
                #enum_ident::#variant_ident(packet) => {
                    packet.encode(&mut packet_writer, protocol_version)?;
                    let packet_bytes = packet_writer.into_inner();
                    let packet_id: u8 = match packets_version {
                        #(#report_arms)*
                        _ => return Err(PacketRegistryEncodeError::UnsupportedPacket(protocol_version, String::from(#packet_name))),
                    };
                    RawPacket::from_bytes(packet_id, &packet_bytes)
                }
            }
        });

    quote! {
        pub fn encode_packet(self, protocol_version: ProtocolVersion) -> Result<RawPacket, PacketRegistryEncodeError> {
            let packets_version = protocol_version.packets().version_number();
            let mut packet_writer = BinaryWriter::new();
            let raw_packet = match self {
                #(#variant_arms)*
                _ => return Err(PacketRegistryEncodeError::CannotBeEncoded),
            };
            Ok(raw_packet)
        }
    }
}

fn capitalize_first(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}
