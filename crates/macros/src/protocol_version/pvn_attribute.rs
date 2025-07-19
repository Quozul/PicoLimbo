use syn::parse::{Parse, ParseStream};
use syn::{Error, Ident, LitStr, Result, Token};

/// Parses the `#[pvn(reports = "...", data = "...")]` attribute.
pub struct PvnAttribute {
    pub reports: Option<LitStr>,
    pub data: Option<LitStr>,
}

impl Parse for PvnAttribute {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut reports: Option<LitStr> = None;
        let mut data: Option<LitStr> = None;

        if input.is_empty() {
            return Ok(PvnAttribute { reports, data });
        }

        let mut parse_kv = |input: ParseStream| -> Result<()> {
            let ident: Ident = input.parse()?;
            input.parse::<Token![=]>()?;
            let value: LitStr = input.parse()?;

            if ident == "reports" {
                if reports.is_some() {
                    return Err(Error::new(ident.span(), "duplicate `reports` field"));
                }
                reports = Some(value);
            } else if ident == "data" {
                if data.is_some() {
                    return Err(Error::new(ident.span(), "duplicate `data` field"));
                }
                data = Some(value);
            } else {
                return Err(Error::new(
                    ident.span(),
                    "expected either `reports` or `data`",
                ));
            }
            Ok(())
        };

        parse_kv(input)?;
        while input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
            parse_kv(input)?;
        }

        Ok(PvnAttribute { reports, data })
    }
}
