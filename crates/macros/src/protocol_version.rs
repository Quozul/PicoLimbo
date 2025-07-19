use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, LitInt, LitStr, Token, parse::Parse, parse_macro_input};

struct PvnAttribute {
    pvn: LitInt,
    reports: Option<LitStr>,
    data: Option<LitStr>,
}

impl Parse for PvnAttribute {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        // First parse the required integer argument.
        let pvn = input.parse::<LitInt>()?;

        // Prepare optional fields.
        let mut reports: Option<LitStr> = None;
        let mut data: Option<LitStr> = None;

        // Parse additional fields regardless of order.
        while input.peek(Token![,]) {
            // Consume the comma.
            let _comma: Token![,] = input.parse()?;

            // Parse the field name.
            let ident: syn::Ident = input.parse()?;
            input.parse::<Token![=]>()?;

            // Match the field name and assign the value.
            if ident == "reports" {
                if reports.is_some() {
                    return Err(syn::Error::new(ident.span(), "duplicate `reports` field"));
                }
                reports = Some(input.parse::<LitStr>()?);
            } else if ident == "data" {
                if data.is_some() {
                    return Err(syn::Error::new(ident.span(), "duplicate `data` field"));
                }
                data = Some(input.parse::<LitStr>()?);
            } else {
                return Err(syn::Error::new(
                    ident.span(),
                    "expected either `reports` or `data`",
                ));
            }
        }

        Ok(PvnAttribute { pvn, reports, data })
    }
}

pub fn expand_protocol_version_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    // Get the identifier of the enum (e.g. ProtocolVersion)
    let enum_ident = input.ident;

    // Ensure the input is an enum.
    let data_enum = match input.data {
        syn::Data::Enum(data) => data,
        _ => {
            return syn::Error::new_spanned(
                enum_ident,
                "ProtocolVersion can only be derived for enums",
            )
            .to_compile_error()
            .into();
        }
    };

    // Prepare vectors for the match arms for each generated implementation.
    let mut display_arms = Vec::new();
    let mut humanize_arms = Vec::new();
    let mut from_str_arms = Vec::new();
    let mut from_i32_arms = Vec::new();
    let mut version_number_arms = Vec::new();
    let mut reports_arms = Vec::new();
    let mut data_arms = Vec::new();
    let mut pvn_values = Vec::new();

    // Iterate over all variants of the enum.
    for variant in data_enum.variants.iter() {
        let variant_ident = &variant.ident;

        // Look for the #[pvn(...)] attribute.
        let mut pvn_attr: Option<PvnAttribute> = None;
        for attr in &variant.attrs {
            if attr.path().is_ident("pvn") {
                pvn_attr = Some(match attr.parse_args::<PvnAttribute>() {
                    Ok(parsed) => parsed,
                    Err(err) => return err.to_compile_error().into(),
                });
                break;
            }
        }
        let pvn_attr = match pvn_attr {
            Some(val) => val,
            None => {
                return syn::Error::new_spanned(
                    variant,
                    "Missing #[pvn(...)] attribute on variant",
                )
                .to_compile_error()
                .into();
            }
        };

        // Parse the integer literal to an i32.
        let pvn_int = match pvn_attr.pvn.base10_parse::<i32>() {
            Ok(num) => num,
            Err(err) => return err.to_compile_error().into(),
        };
        pvn_values.push((pvn_int, variant_ident));

        // For Display, we want to output the variant name as a string.
        display_arms.push(quote! {
            #enum_ident::#variant_ident => f.write_str(stringify!(#variant_ident))
        });

        let pvn_lit = pvn_attr.pvn;
        from_i32_arms.push(quote! {
            #pvn_lit => #enum_ident::#variant_ident
        });

        version_number_arms.push(quote! {
            #enum_ident::#variant_ident => #pvn_lit
        });

        // For humanize, strip the leading 'V' and replace underscores with dots.
        let variant_name = variant_ident.to_string();
        let humanized = variant_name
            .strip_prefix('V')
            .unwrap_or(variant_name.as_str())
            .replace('_', ".");
        let humanized_lit = LitStr::new(&humanized, variant_ident.span());
        humanize_arms.push(quote! {
            #enum_ident::#variant_ident => #humanized_lit
        });

        from_str_arms.push(quote! {
            #humanized_lit => Ok(#enum_ident::#variant_ident)
        });

        let reports_value = pvn_attr
            .reports
            .unwrap_or_else(|| LitStr::new(&variant_ident.to_string(), variant_ident.span()));
        reports_arms.push(quote! {
            #enum_ident::#variant_ident => #reports_value
        });

        let data_value = pvn_attr
            .data
            .unwrap_or_else(|| LitStr::new(&variant_ident.to_string(), variant_ident.span()));
        data_arms.push(quote! {
            #enum_ident::#variant_ident => #data_value
        });
    }

    // Determine the smallest and largest pvn values and their associated variants.
    let (min_value, min_variant_ident) = pvn_values
        .iter()
        .min_by_key(|(val, _)| *val)
        .map(|(v, ident)| (v, ident))
        .expect("At least one variant is required");
    let (max_value, max_variant_ident) = pvn_values
        .iter()
        .max_by_key(|(val, _)| *val)
        .map(|(v, ident)| (v, ident))
        .expect("At least one variant is required");

    let expanded = quote! {
        impl std::fmt::Display for #enum_ident {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    #(#display_arms),*
                }
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

        impl FromStr for ProtocolVersion {
            type Err = std::io::Error;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    #(#from_str_arms),*,
                    _ => Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid version")),
                }
            }
        }

        impl #enum_ident {
            pub fn version_number(&self) -> u32 {
                match self {
                    #(#version_number_arms),*
                }
            }

            pub fn humanize(&self) -> &'static str {
                match self {
                    #(#humanize_arms),*
                }
            }

            pub fn reports(&self) -> &'static str {
                match self {
                    #(#reports_arms),*
                }
            }

            pub fn data(&self) -> &'static str {
                match self {
                    #(#data_arms),*
                }
            }

            pub fn latest() -> Self {
                #enum_ident::#max_variant_ident
            }

            pub fn oldest() -> Self {
                #enum_ident::#min_variant_ident
            }
        }
    };

    TokenStream::from(expanded)
}
