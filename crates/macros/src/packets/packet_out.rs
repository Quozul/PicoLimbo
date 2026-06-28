extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};
use crate::packets::common::{get_named_fields, get_pvn_attribute};

pub fn expand_parse_out_packet_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let fields = get_named_fields(&input);

    let field_parsers = fields.iter().map(|field| {
        let field_name = &field.ident;
        let version_range = get_pvn_attribute(field);

        if let Some(version_range) = version_range {
            quote! {
                if (#version_range).contains(&protocol_version.version_number()) {
                    self.#field_name.encode(writer, protocol_version)?;
                }
            }
        } else {
            quote! {
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
