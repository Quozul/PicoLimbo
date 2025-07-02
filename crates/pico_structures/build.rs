use proc_macro2::TokenStream;
use quote::quote;
use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;

#[derive(Deserialize, Debug)]
struct BlockState {
    #[serde(default)]
    default: bool,
    id: u32,
    #[serde(default)]
    properties: HashMap<String, String>,
}

#[derive(Deserialize, Debug)]
struct Block {
    states: Vec<BlockState>,
}

fn main() {
    println!("cargo:rerun-if-changed=data/");

    let json_content = fs::read_to_string("data/blocks.json").expect("Failed to read blocks.json");
    let blocks: HashMap<String, Block> =
        serde_json::from_str(&json_content).expect("Failed to parse blocks.json");

    let generated_code = generate_block_code(&blocks);

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("blocks.rs");
    fs::write(&dest_path, generated_code.to_string())
        .expect("Failed to write generated blocks code");
}

fn generate_block_code(blocks: &HashMap<String, Block>) -> TokenStream {
    let mut sorted_blocks: Vec<(&String, &Block)> = blocks.iter().collect();
    sorted_blocks.sort_by_key(|(id, _)| id.as_str());

    let block_data_entries = sorted_blocks.iter().map(|(_, block)| {
        let state_entries = block.states.iter().map(|state| {
            let state_id = state.id as i32;
            let is_default = state.default;

            let mut sorted_properties: Vec<(&String, &String)> = state.properties.iter().collect();
            sorted_properties.sort_by_key(|(k, _)| k.as_str());

            let prop_entries = sorted_properties.iter().map(|(k, v)| {
                quote! { (#k, #v) }
            });

            quote! {
                StaticBlockState {
                    id: #state_id,
                    is_default: #is_default,
                    properties: &[#(#prop_entries),*],
                }
            }
        });

        quote! {
            StaticBlock {
                states: &[#(#state_entries),*],
            }
        }
    });

    let search_match_arms = sorted_blocks.iter().enumerate().map(|(i, (id, _))| {
        quote! { #id => find_state_id(&ALL_BLOCKS[#i], &query_props) }
    });

    quote! {
        #[derive(Debug, Clone, Copy)]
        struct StaticBlockState {
            id: i32,
            is_default: bool,
            properties: &'static [(&'static str, &'static str)],
        }

        #[derive(Debug, Clone, Copy)]
        struct StaticBlock {
            states: &'static [StaticBlockState],
        }

        static ALL_BLOCKS: &[StaticBlock] = &[
            #(#block_data_entries),*
        ];

        #[derive(Debug, Default)]
        pub struct SearchState {
            block_name: Option<String>,
            properties: Vec<(String, String)>,
        }

        impl SearchState {
            pub fn new() -> Self {
                Self::default()
            }

            pub fn block_name(&mut self, name: impl ToString) -> &mut Self {
                self.block_name = Some(name.to_string());
                self
            }

            pub fn property(&mut self, name: impl ToString, value: impl ToString) -> &mut Self {
                self.properties.push((name.to_string(), value.to_string()));
                self
            }

            pub fn build(&mut self) -> Option<i32> {
                let block_name = self.block_name.as_ref()?;

                self.properties.sort_by(|(a, _), (b, _)| a.cmp(b));

                let query_props: Vec<(&str, &str)> = self.properties
                    .iter()
                    .map(|(k, v)| (k.as_str(), v.as_str()))
                    .collect();

                match block_name.as_str() {
                    #(#search_match_arms,)*
                    _ => None,
                }
            }
        }

        #[inline]
        fn find_state_id(block: &StaticBlock, query_props: &[(&str, &str)]) -> Option<i32> {
            if query_props.is_empty() {
                for state in block.states.iter() {
                    if state.is_default {
                        return Some(state.id);
                    }
                }
                return block.states.first().map(|s| s.id);
            }

            for state in block.states.iter() {
                if state.properties.len() == query_props.len() && state.properties == query_props {
                    return Some(state.id);
                }
            }
            None
        }
    }
}
