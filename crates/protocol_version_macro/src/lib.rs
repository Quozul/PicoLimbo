use proc_macro::TokenStream;

mod expand;
mod find_min_max_variants;
mod parsed_variant;
mod pvn_attribute;

#[proc_macro_derive(Pvn, attributes(pvn, default))]
pub fn protocol_version_derive(input: TokenStream) -> TokenStream {
    expand::expand_protocol_version_derive(input)
}
