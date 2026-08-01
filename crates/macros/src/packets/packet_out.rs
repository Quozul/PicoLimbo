extern crate proc_macro;
use crate::packets::common::VersionConstraint;
use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

pub fn expand_parse_out_packet_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let fields = if let syn::Data::Struct(data) = &input.data {
        if let syn::Fields::Named(fields) = &data.fields {
            &fields.named
        } else {
            unimplemented!()
        }
    } else {
        unimplemented!()
    };

    let field_parsers = fields.iter().map(|field| {
        let field_name = &field.ident;
        let version_check = field
            .attrs
            .iter()
            .find_map(VersionConstraint::from_attribute)
            .map(|c| c.generate_check())
            .unwrap_or_else(|| quote! { true });

        quote! {
            if #version_check {
                self.#field_name.encode(writer, protocol_version)?;
            }
        }
    });

    let expanded = quote! {
        impl EncodePacket for #name {
            fn encode(&self, writer: &mut BinaryWriter, protocol_version: ProtocolVersion) -> Result<(), BinaryWriterError> {
                #(#field_parsers)*
                Ok(())
            }
        }
    };

    TokenStream::from(expanded)
}
