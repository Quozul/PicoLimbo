use syn::parse::{Parse, ParseStream};
use syn::{Error, Ident, Result, Token};

/// Parses the `#[pvn(reports = "...", data = "...")]` attribute.
pub struct PvnAttribute {
    pub packets: Option<Ident>,
    pub data: Option<Ident>,
}

impl Parse for PvnAttribute {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut packets: Option<Ident> = None;
        let mut data: Option<Ident> = None;

        if input.is_empty() {
            return Ok(PvnAttribute { packets, data });
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
            } else {
                return Err(Error::new(
                    ident.span(),
                    "expected either `packets` or `data`",
                ));
            }
            Ok(())
        };

        parse_kv(input)?;
        while input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
            parse_kv(input)?;
        }

        Ok(PvnAttribute { packets, data })
    }
}
