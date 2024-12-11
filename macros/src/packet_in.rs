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
        quote! {
            let #field_name = <#field_type as DeserializePacketData>::decode(bytes, &mut index).map_err(|_| DecodePacketError)?;
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
            fn decode(bytes: &[u8]) -> Result<Self, DecodePacketError> {
                let mut index = 0;
                #(#field_parsers)*

                Ok(Self {
                    #(#field_initializers)*
                })
            }
        }
    };

    TokenStream::from(expanded)
}
