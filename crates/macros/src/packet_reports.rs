use proc_macro::TokenStream;
use quote::{format_ident, quote};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use syn::parse::{Parse, ParseStream};
use syn::{Data, DeriveInput, Error, Fields, Ident, LitStr, Token, parse_macro_input};

#[derive(Debug, Deserialize)]
struct JsonReport {
    handshake: JsonState,
    status: JsonState,
    login: JsonState,
    play: JsonState,
}

#[derive(Debug, Deserialize, Default)]
struct JsonState {
    // Use serde(default) to handle cases where a bound might be missing in the JSON.
    #[serde(default)]
    serverbound: HashMap<String, JsonPacket>,
    #[serde(default)]
    clientbound: HashMap<String, JsonPacket>,
}

#[derive(Debug, Deserialize)]
struct JsonPacket {
    protocol_id: i32,
}

// Struct to hold the parsed information from our enum variants
struct PacketVariantInfo {
    variant_ident: Ident,
    packet_type: syn::Path,
    state: String,
    // bound is always "serverbound" in this context, but we parse it for completeness
    bound: String,
    name: String,
}

pub fn packet_report_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let enum_ident = &input.ident;

    let variants = parse_enum_variants(&input.data);
    let protocol_data = load_all_protocol_data();

    // Generate each implementation by filtering the variants based on their bound.
    let decode_impl = generate_decode_impl(&variants, &protocol_data);
    let encode_impl = generate_encode_impl(enum_ident, &variants, &protocol_data);

    let expanded = quote! {
        // ... (Error enum and From impls are the same) ...
        // For runnable code
        #[derive(Debug)]
        pub enum PacketRegistryError {
            UnknownPacket(String),
            Decode(BinaryReaderError),
            Encode(BinaryWriterError),
        }
        impl From<BinaryReaderError> for PacketRegistryError { fn from(e: BinaryReaderError) -> Self { Self::Decode(e) } }
        impl From<BinaryWriterError> for PacketRegistryError { fn from(e: BinaryWriterError) -> Self { Self::Encode(e) } }

        impl #enum_ident {
            #decode_impl
            #encode_impl
        }
    };

    TokenStream::from(expanded)
}

// --- Helper Functions for the Macro ---

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
                panic!("No #[protocol_id] attribute found")
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

/// Parses the `#[pvn(reports = "...", data = "...")]` attribute.
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

fn load_all_protocol_data() -> HashMap<String, JsonReport> {
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
                let report: JsonReport = serde_json::from_str(&content)
                    .unwrap_or_else(|e| panic!("Failed to parse {:?}: {}", report_path, e));
                all_data.insert(version_name, report);
            }
        }
    }
    all_data
}

// --- Code Generation Functions ---

fn generate_decode_impl(
    variants: &[PacketVariantInfo],
    protocol_data: &HashMap<String, JsonReport>,
) -> proc_macro2::TokenStream {
    let report_arms = protocol_data.iter().map(|(version_name, report)| {
        let state_arms = variants
            .iter()
            // 1. Filter for SERVERBOUND packets only
            .filter(|v| v.bound == "serverbound")
            .map(|variant_info| {
                let state_str = &variant_info.state;
                let packet_name = &variant_info.name;

                let json_state = get_json_state(report, state_str);

                // 2. Look in the `serverbound` map
                if let Some(packet_data) = json_state.serverbound.get(packet_name) {
                    let id = packet_data.protocol_id;
                    let state_ident = format_ident!("{}", capitalize_first(state_str));
                    let packet_type = &variant_info.packet_type;
                    let variant_ident = &variant_info.variant_ident;

                    quote! {
                        (State::#state_ident, #id) => {
                            let packet = <#packet_type>::decode(payload, protocol_version)?;
                            return Ok(Self::#variant_ident(packet));
                        }
                    }
                } else {
                    quote! {}
                }
            });

        quote! {
            #version_name => {
                let state_string = state.to_string();
                match (state, packet_id) {
                    #(#state_arms)*
                    _ => return Err(PacketRegistryError::UnknownPacket(format!("Unknown serverbound packet id {} in state {} for version {}", packet_id, state_string, #version_name))),
                }
            }
        }
    });

    quote! {
        pub fn decode_packet(
            protocol_version: ProtocolVersion,
            state: State,
            packet_id: i32,
            payload: &mut BinaryReader,
        ) -> Result<Self, PacketRegistryError> {
            let report_name = protocol_version.reports();
            match report_name {
                #(#report_arms)*
                _ => return Err(PacketRegistryError::UnknownPacket(format!("Unsupported protocol report: {}", report_name))),
            }
        }
    }
}

fn generate_encode_impl(
    enum_ident: &Ident,
    variants: &[PacketVariantInfo],
    protocol_data: &HashMap<String, JsonReport>,
) -> proc_macro2::TokenStream {
    let variant_arms = variants
        .iter()
        // 1. Filter for CLIENTBOUND packets only
        .filter(|v| v.bound == "clientbound")
        .map(|variant_info| {
            let variant_ident = &variant_info.variant_ident;
            let packet_name = &variant_info.name;
            let state_str = &variant_info.state;

            let report_arms = protocol_data.iter().map(|(version_name, report)| {
                let json_state = get_json_state(report, state_str);

                // 2. Look in the `clientbound` map
                if let Some(packet_data) = json_state.clientbound.get(packet_name) {
                    let id = packet_data.protocol_id as u8;
                    quote! {
                        #version_name => #id,
                    }
                } else {
                    quote! {}
                }
            });

            quote! {
                // Notice the variant name is now inside the match self arm
                #enum_ident::#variant_ident(packet) => {
                    let packet_id = match report_name {
                        #(#report_arms)*
                        _ => return Err(PacketRegistryError::UnknownPacket(format!("Could not find packet id for {} in report {}", #packet_name, report_name))),
                    };
                    writer.write(&packet_id)?;
                    packet.encode(&mut writer, protocol_version)?;
                }
            }
        });

    quote! {
        // The `state` parameter is removed as it's implicit from the enum variant being encoded.
        pub fn encode_packet(&self, protocol_version: ProtocolVersion) -> Result<Vec<u8>, PacketRegistryError> {
            let mut writer = BinaryWriter::new();
            let report_name = protocol_version.reports();

            match self {
                #(#variant_arms)*
                // This match is no longer exhaustive for all enum variants, so we add a fallback arm.
                // This arm will catch any `serverbound` packets passed to `encode_packet`.
                _ => return Err(PacketRegistryError::UnknownPacket("Attempted to encode a serverbound packet.".to_string())),
            }
            Ok(writer.into_inner())
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

// Helper to reduce repetition
fn get_json_state<'a>(report: &'a JsonReport, state_str: &str) -> &'a JsonState {
    match state_str {
        "handshake" => &report.handshake,
        "status" => &report.status,
        "login" => &report.login,
        "play" => &report.play,
        _ => panic!("Unknown state in attribute: {}", state_str),
    }
}
