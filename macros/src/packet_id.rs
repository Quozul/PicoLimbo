extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, Lit, parse_macro_input};

pub fn expand_packet_id(attr: TokenStream, item: TokenStream) -> TokenStream {
    let lit = parse_macro_input!(attr as Lit);
    let input = parse_macro_input!(item as DeriveInput);
    let struct_name = &input.ident;

    let packet_id = if let Lit::Int(lit_int) = lit {
        lit_int.base10_parse::<u8>().unwrap()
    } else {
        panic!("Expected an integer literal for packet_id");
    };

    let expanded = quote! {
        #[allow(dead_code)]
        #input

        impl PacketId for #struct_name {
            const PACKET_ID: u8 = #packet_id;

            fn get_packet_id(&self) -> u8 {
                Self::PACKET_ID
            }
        }
    };

    TokenStream::from(expanded)
}
