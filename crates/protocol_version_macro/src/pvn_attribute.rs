use syn::parse::{Parse, ParseStream};
use syn::{Error, Ident, LitStr, Result, Token};

/// Parses the `#[pvn(packets = "...", data = "...", known_packs = [...])]` attribute.
pub struct PvnAttribute {
    pub packets: Option<Ident>,
    pub data: Option<Ident>,
    pub known_packs: Vec<String>,
}

impl Parse for PvnAttribute {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut packets: Option<Ident> = None;
        let mut data: Option<Ident> = None;
        let mut known_packs: Vec<String> = Vec::new();

        if input.is_empty() {
            return Ok(PvnAttribute {
                packets,
                data,
                known_packs,
            });
        }

        let mut parse_kv = |input: ParseStream| -> Result<()> {
            let ident: Ident = input.parse()?;
            input.parse::<Token![=]>()?;

            if ident == "packets" {
                if packets.is_some() {
                    return Err(Error::new(ident.span(), "duplicate `packets` field"));
                }
                let value: Ident = input.parse()?;
                packets = Some(value);
            } else if ident == "data" {
                if data.is_some() {
                    return Err(Error::new(ident.span(), "duplicate `data` field"));
                }
                let value: Ident = input.parse()?;
                data = Some(value);
            } else if ident == "known_packs" {
                if !known_packs.is_empty() {
                    return Err(Error::new(ident.span(), "duplicate `known_packs` field"));
                }
                let content;
                syn::bracketed!(content in input);
                while !content.is_empty() {
                    let s: LitStr = content.parse()?;
                    known_packs.push(s.value());
                    if content.peek(Token![,]) {
                        content.parse::<Token![,]>()?;
                    }
                }
            } else {
                return Err(Error::new(
                    ident.span(),
                    "expected `packets`, `data`, or `known_packs`",
                ));
            }
            Ok(())
        };

        parse_kv(input)?;
        while input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
            parse_kv(input)?;
        }

        Ok(PvnAttribute {
            packets,
            data,
            known_packs,
        })
    }
}
