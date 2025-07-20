extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, parse_macro_input};

pub fn expand_parse_packet_in_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let fields = if let Data::Struct(data) = &input.data {
        if let Fields::Named(fields) = &data.fields {
            &fields.named
        } else {
            unimplemented!()
        }
    } else {
        unimplemented!()
    };

    let field_parsers = fields.iter().map(|field| {
        let field_name = &field.ident;
        let field_type = &field.ty;
        let version_range = field.attrs.iter().find_map(|attr| {
            if attr.path().is_ident("pvn") {
                Some(attr.parse_args::<syn::Expr>().unwrap())
            } else {
                None
            }
        });

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
