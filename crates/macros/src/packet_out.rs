extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, parse_macro_input};

pub fn expand_parse_out_packet_derive(input: TokenStream) -> TokenStream {
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
        let version_range = field.attrs.iter().find_map(|attr| {
            if attr.path().is_ident("pvn") {
                Some(attr.parse_args::<syn::Expr>().unwrap())
            } else {
                None
            }
        });

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
