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
        quote! {
            self.#field_name.encode(&mut bytes)?;
        }
    });

    let expanded = quote! {
        impl EncodePacket for #name {
            fn encode(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
                let mut bytes = Vec::new();
                #(#field_parsers)*
                Ok(bytes)
            }
        }
    };

    TokenStream::from(expanded)
}
