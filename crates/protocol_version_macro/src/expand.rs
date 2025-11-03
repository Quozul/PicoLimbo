use crate::find_min_max_variants::find_min_max_variants;
use crate::parsed_variant::ParsedVariant;
use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Error, parse_macro_input};

pub fn expand_protocol_version_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let enum_ident = &input.ident;

    let data_enum = match &input.data {
        Data::Enum(data) => data,
        _ => {
            return Error::new_spanned(enum_ident, "ProtocolVersion can only be derived for enums")
                .to_compile_error()
                .into();
        }
    };

    // Parse all variants into our structured format.
    let parsed_variants: Vec<ParsedVariant> = match data_enum
        .variants
        .iter()
        .map(ParsedVariant::from_variant)
        .collect()
    {
        Ok(variants) => variants,
        Err(err) => return err.to_compile_error().into(),
    };

    // Find the min and max "real" versions.
    let (min_variant_ident, max_variant_ident) =
        match find_min_max_variants(&parsed_variants, enum_ident.span()) {
            Ok(idents) => idents,
            Err(err) => return err.to_compile_error().into(),
        };

    let max_value = ParsedVariant::from_variant(
        data_enum
            .variants
            .iter()
            .find(|v| v.ident == *max_variant_ident)
            .unwrap(),
    )
    .unwrap()
    .discriminant_value;

    let min_value = ParsedVariant::from_variant(
        data_enum
            .variants
            .iter()
            .find(|v| v.ident == *min_variant_ident)
            .unwrap(),
    )
    .unwrap()
    .discriminant_value;

    // Generate the code for each impl block by iterating over `parsed_variants`.
    let display_arms = parsed_variants.iter().map(|v| {
        let variant_ident = v.ident;
        quote! { #enum_ident::#variant_ident => f.write_str(stringify!(#variant_ident)) }
    });

    let from_i32_arms = parsed_variants.iter().map(|v| {
        let variant_ident = v.ident;
        let discriminant_expr = v.discriminant_expr;
        quote! { #discriminant_expr => Ok(#enum_ident::#variant_ident) }
    });

    let humanize_arms = parsed_variants.iter().map(|v| {
        let variant_ident = v.ident;
        let humanized_lit = &v.humanized_string;
        quote! { #enum_ident::#variant_ident => #humanized_lit }
    });

    let from_str_arms = parsed_variants.iter().map(|v| {
        let variant_ident = v.ident;
        let variant_string = v.ident.to_string();
        quote! { #variant_string => Ok(#enum_ident::#variant_ident) }
    });

    let reports_arms = parsed_variants.iter().map(|v| {
        let variant_ident = v.ident;
        let value = &v.reports;
        quote! { #enum_ident::#variant_ident => #enum_ident::#value }
    });

    let data_arms = parsed_variants.iter().map(|v| {
        let variant_ident = v.ident;
        let value = &v.data;
        quote! { #enum_ident::#variant_ident => #enum_ident::#value }
    });

    let all_versions_arms = parsed_variants.iter().map(|v| {
        let variant_ident = v.ident;
        quote! { #enum_ident::#variant_ident }
    });

    // Assemble the final TokenStream.
    let expanded = quote! {
        impl std::fmt::Display for #enum_ident {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self { #(#display_arms),* }
            }
        }

        #[derive(thiserror::Error, Debug)]
        #[error("Unsupported protocol version {0}")]
        pub struct InvalidProtocolVersion(i32);

        impl TryFrom<i32> for #enum_ident {
            type Error = InvalidProtocolVersion;

            fn try_from(value: i32) -> Result<Self, Self::Error> {
                match value {
                    #(#from_i32_arms),*,
                    _ => Err(InvalidProtocolVersion(value)),
                }
            }
        }

        impl std::str::FromStr for #enum_ident {
            type Err = std::io::Error;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    #(#from_str_arms),*,
                    _ => Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid version")),
                }
            }
        }

        impl #enum_ident {
            pub const ALL_VERSION: &'static [ProtocolVersion] = &[#(#all_versions_arms),*];

            pub fn from(value: i32) -> Self {
                Self::try_from(value).unwrap_or_else(|_| {
                    if value > #max_value {
                        #enum_ident::#max_variant_ident
                    } else if value < #min_value {
                        #enum_ident::#min_variant_ident
                    } else {
                        Self::default()
                    }
                })
            }

            /// Returns the protocol version number by casting the enum to an i32.
            pub fn version_number(&self) -> i32 {
                *self as i32
            }

            /// Returns the human-readable version string (e.g., "1.18.2").
            pub fn humanize(&self) -> &'static str {
                match self { #(#humanize_arms),* }
            }

            /// Returns the protocol version this version reports as.
            pub fn reports(&self) -> ProtocolVersion {
                match self { #(#reports_arms),* }
            }

            /// Returns the protocol version this version gets its data from.
            pub fn data(&self) -> ProtocolVersion {
                match self { #(#data_arms),* }
            }

            /// Returns the latest real protocol version (ignores special values like `Any`).
            pub fn latest() -> Self {
                #enum_ident::#max_variant_ident
            }

            /// Returns the oldest real protocol version (ignores special values like `Any`).
            pub fn oldest() -> Self {
                #enum_ident::#min_variant_ident
            }
        }
    };

    TokenStream::from(expanded)
}
