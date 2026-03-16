use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, LitStr, parse_macro_input};

pub fn expand(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let data = match &input.data {
        syn::Data::Enum(data) => data,
        _ => panic!("RegistryKeys can only be derived for enums"),
    };

    let mut all_registries = Vec::new();
    let mut id_arms = Vec::new();
    let mut is_mandatory_arms = Vec::new();
    let mut min_version_arms = Vec::new();
    let mut is_root_arms = Vec::new();

    for variant in &data.variants {
        let variant_name = &variant.ident;
        let pat = match variant.fields {
            syn::Fields::Named(_) => quote! { Self::#variant_name { .. } },
            syn::Fields::Unnamed(_) => quote! { Self::#variant_name(..) },
            syn::Fields::Unit => quote! { Self::#variant_name },
        };

        let mut is_root = false;
        let mut is_custom = false;
        let mut id = None;
        let mut min_version: Option<syn::Path> = None;
        let mut is_mandatory = false;

        for attr in &variant.attrs {
            if attr.path().is_ident("registry") {
                attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("root") {
                        is_root = true;
                        Ok(())
                    } else if meta.path.is_ident("custom") {
                        is_custom = true;
                        Ok(())
                    } else if meta.path.is_ident("id") {
                        let value = meta.value()?;
                        let lit: LitStr = value.parse()?;
                        let s = lit.value();
                        let s = s.strip_prefix("minecraft:").unwrap_or(&s).to_string();
                        id = Some(s);
                        Ok(())
                    } else if meta.path.is_ident("min_version") {
                        let value = meta.value()?;
                        min_version = Some(value.parse()?);
                        Ok(())
                    } else if meta.path.is_ident("is_mandatory") {
                        let value = meta.value()?;
                        let lit: syn::LitBool = value.parse()?;
                        is_mandatory = lit.value;
                        Ok(())
                    } else {
                        Err(meta.error("unrecognized registry attribute"))
                    }
                })
                .unwrap_or_else(|e| {
                    panic!(
                        "failed to parse registry attribute for {}: {}",
                        variant_name, e
                    )
                });
            }
        }

        if is_root {
            id_arms.push(quote! { #pat => Identifier::vanilla_unchecked("root") });
            is_mandatory_arms.push(quote! { #pat => false });
            is_root_arms.push(quote! { #pat => true });
            min_version_arms.push(quote! { #pat => None });
        } else if is_custom {
            id_arms.push(quote! { Self::#variant_name(identifier) => identifier.clone() });
            is_mandatory_arms.push(quote! { #pat => false });
            is_root_arms.push(quote! { #pat => false });
            min_version_arms.push(quote! { #pat => None });
        } else {
            let id_str = id.unwrap_or_else(|| {
                panic!(
                    "variant {} missing id attribute in #[registry]",
                    variant_name
                )
            });
            id_arms.push(quote! { #pat => Identifier::vanilla_unchecked(#id_str) });
            is_mandatory_arms.push(quote! { #pat => #is_mandatory });
            is_root_arms.push(quote! { #pat => false });

            if let Some(mv) = min_version {
                min_version_arms.push(quote! { #pat => Some(ProtocolVersion::#mv) });
            } else {
                min_version_arms.push(quote! { #pat => None });
            }

            // Exclude everything that is root or custom, include the rest into ALL_REGISTRIES
            all_registries.push(quote! { Self::#variant_name });
        }
    }

    let expanded = quote! {
        impl #name {
            pub const ALL_REGISTRIES: &[Self] = &[
                #(#all_registries,)*
            ];

            #[must_use]
            pub fn id(&self) -> Identifier {
                match self {
                    #(#id_arms,)*
                }
            }

            #[must_use]
            pub const fn is_mandatory(&self) -> bool {
                match self {
                    #(#is_mandatory_arms,)*
                }
            }

            #[must_use]
            pub const fn is_root(&self) -> bool {
                match self {
                    #(#is_root_arms,)*
                }
            }

            #[must_use]
            pub fn get_tag_path(&self) -> String {
                format!("tags/{}", self.id().thing)
            }

            #[must_use]
            pub const fn get_minimum_version(&self) -> Option<ProtocolVersion> {
                match self {
                    #(#min_version_arms,)*
                }
            }
        }
    };

    TokenStream::from(expanded)
}
