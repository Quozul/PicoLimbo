use proc_macro::TokenStream;

mod packet_id;
mod packet_in;
mod packet_out;

#[proc_macro_attribute]
pub fn packet_id(attr: TokenStream, item: TokenStream) -> TokenStream {
    packet_id::expand_packet_id(attr, item)
}

#[proc_macro_derive(PacketIn, attributes(pvn))]
pub fn parse_packet_in_derive(input: TokenStream) -> TokenStream {
    packet_in::expand_parse_packet_in_derive(input)
}

#[proc_macro_derive(PacketOut, attributes(pvn))]
pub fn parse_out_packet_derive(input: TokenStream) -> TokenStream {
    packet_out::expand_parse_out_packet_derive(input)
}
