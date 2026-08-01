use proc_macro::TokenStream;

mod packet_reports;
mod packets;
mod registry_keys;

#[proc_macro_derive(PacketIn, attributes(protocol_version))]
pub fn parse_packet_in_derive(input: TokenStream) -> TokenStream {
    packets::packet_in::expand_parse_packet_in_derive(input)
}

#[proc_macro_derive(PacketOut, attributes(protocol_version))]
pub fn parse_out_packet_derive(input: TokenStream) -> TokenStream {
    packets::packet_out::expand_parse_out_packet_derive(input)
}

#[proc_macro_derive(PacketReport, attributes(protocol_id))]
pub fn packet_report_derive(input: TokenStream) -> TokenStream {
    packet_reports::packet_report_derive(input)
}

#[proc_macro_derive(RegistryKeys, attributes(registry))]
pub fn registry_keys_derive(input: TokenStream) -> TokenStream {
    registry_keys::expand(input)
}
