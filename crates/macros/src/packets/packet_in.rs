extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};
use crate::packets::common::{get_named_fields, get_pvn_attribute};

pub fn expand_parse_packet_in_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let fields = get_named_fields(&input);

    let field_parsers = fields.iter().map(|field| {
        let field_name = &field.ident;
        let field_type = &field.ty;
        let version_range = get_pvn_attribute(field);

        if let Some(version_range) = version_range {
            quote! {
                let #field_name: #field_type = if (#version_range).contains(&protocol_version.version_number()) {
                    <#field_type as DecodePacket>::decode(reader, protocol_version)?
                } else {
                    Default::default()
                };
            }
        } else {
            quote! {
                let #field_name = <#field_type as DecodePacket>::decode(reader, protocol_version)?;
            }
        }
    });

    let field_initializers = fields.iter().map(|field| {
        let field_name = &field.ident;
        quote! {
            #field_name,
        }
    });

    let expanded = quote! {
        impl DecodePacket for #name {
            fn decode(reader: &mut BinaryReader, protocol_version: ProtocolVersion) -> Result<Self, BinaryReaderError> {
                #(#field_parsers)*

                Ok(Self {
                    #(#field_initializers)*
                })
            }
        }
    };

    TokenStream::from(expanded)
}
