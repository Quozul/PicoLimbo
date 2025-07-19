use crate::protocol_version::find_min_max_variants::find_min_max_variants;
use crate::protocol_version::parsed_variant::ParsedVariant;
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
        quote! { #discriminant_expr => #enum_ident::#variant_ident }
    });

    let humanize_arms = parsed_variants.iter().map(|v| {
        let variant_ident = v.ident;
        let humanized_lit = &v.humanized_string;
        quote! { #enum_ident::#variant_ident => #humanized_lit }
    });

    let from_str_arms = parsed_variants.iter().map(|v| {
        let variant_ident = v.ident;
        let humanized_lit = &v.humanized_string;
        quote! { #humanized_lit => Ok(#enum_ident::#variant_ident) }
    });

    let reports_arms = parsed_variants.iter().map(|v| {
        let variant_ident = v.ident;
        let value = &v.reports;
        quote! { #enum_ident::#variant_ident => #value }
    });

    let data_arms = parsed_variants.iter().map(|v| {
        let variant_ident = v.ident;
        let value = &v.data;
        quote! { #enum_ident::#variant_ident => #value }
    });

    // Assemble the final TokenStream.
    let expanded = quote! {
        impl std::fmt::Display for #enum_ident {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self { #(#display_arms),* }
            }
        }

        impl From<i32> for #enum_ident {
            fn from(value: i32) -> Self {
                match value {
                    #(#from_i32_arms),*,
                    v if v > #max_value => #enum_ident::#max_variant_ident,
                    v if v < #min_value => #enum_ident::#min_variant_ident,
                    _ => Self::default(),
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
            /// Returns the protocol version number by casting the enum to an i32.
            pub fn version_number(&self) -> i32 {
                *self as i32
            }

            /// Returns the human-readable version string (e.g., "1.18.2").
            pub fn humanize(&self) -> &'static str {
                match self { #(#humanize_arms),* }
            }

            /// Returns the protocol version this version reports as.
            pub fn reports(&self) -> &'static str {
                match self { #(#reports_arms),* }
            }

            /// Returns the protocol version this version gets its data from.
            pub fn data(&self) -> &'static str {
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
