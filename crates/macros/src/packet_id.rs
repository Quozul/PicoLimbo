extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    DeriveInput, LitStr,
    parse::{Parse, ParseStream},
    parse_macro_input,
};

struct PacketArgs {
    packet_name: String,
}

impl Parse for PacketArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let packet_name_lit: LitStr = input.parse()?;
        Ok(PacketArgs {
            packet_name: packet_name_lit.value(),
        })
    }
}

pub fn expand_packet_id(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as PacketArgs);
    let input = parse_macro_input!(item as DeriveInput);
    let struct_name = &input.ident;

    let packet_name = LitStr::new(&args.packet_name, proc_macro2::Span::call_site());
    let expanded = quote! {
        #[allow(dead_code)]
        #input

        impl Identifiable for #struct_name {
            const PACKET_NAME: &'static str = #packet_name;

            fn get_packet_name(&self) -> &'static str {
                Self::PACKET_NAME
            }
        }
    };

    TokenStream::from(expanded)
}
