use crate::prelude::StringIndexer;
use proc_macro2::TokenStream;
use quote::quote;

impl StringIndexer {
    /// Generates a function with a single `match` statement that takes a string and returns the index
    pub fn codegen(&self) -> TokenStream {
        let match_arms = self.strings.iter().enumerate().map(|(index, string)| {
            let number = index as u16;
            quote! {
                #string => Some(#number),
            }
        });

        quote! {
            fn string_to_index(s: &str) -> Option<u16> {
                match s {
                    #(#match_arms)*
                    _ => None,
                }
            }
        }
    }
}
