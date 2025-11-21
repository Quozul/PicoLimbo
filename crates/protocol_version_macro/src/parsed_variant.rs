use crate::pvn_attribute::PvnAttribute;
use syn::{Error, Expr, Ident, Lit, LitStr, Result, UnOp, Variant};

/// A struct to hold all processed information from a single enum variant.
pub struct ParsedVariant<'a> {
    pub ident: &'a Ident,
    pub discriminant_expr: &'a Expr,
    pub discriminant_value: i32,
    pub humanized_string: LitStr,
    pub packets: Ident,
    pub data: Ident,
}

impl<'a> ParsedVariant<'a> {
    /// Parses a `syn::Variant` into a more structured `ParsedVariant`.
    pub fn from_variant(variant: &'a Variant) -> Result<Self> {
        let discriminant_expr = Self::get_discriminant_expr(variant)?;
        let discriminant_value = Self::parse_discriminant_value(discriminant_expr)?;
        let humanized_string = Self::humanize_variant_name(&variant.ident);

        let pvn_attribute = Self::parse_pvn_attribute(variant)?;

        let packets = pvn_attribute
            .packets
            .unwrap_or_else(|| variant.ident.clone());

        let data = pvn_attribute.data.unwrap_or_else(|| variant.ident.clone());

        Ok(Self {
            ident: &variant.ident,
            discriminant_expr,
            discriminant_value,
            humanized_string,
            packets,
            data,
        })
    }

    /// Extracts the discriminant expression from a variant, erroring if it's missing.
    pub fn get_discriminant_expr(variant: &'a Variant) -> Result<&'a Expr> {
        match &variant.discriminant {
            Some((_, expr)) => Ok(expr),
            None => Err(Error::new_spanned(
                variant,
                "All variants must have an explicit discriminant (e.g., `V1_8 = 47`)",
            )),
        }
    }

    /// Parses an `i32` value from a discriminant expression, supporting literals and negation.
    pub fn parse_discriminant_value(expr: &Expr) -> Result<i32> {
        match expr {
            Expr::Lit(expr_lit) => {
                if let Lit::Int(lit_int) = &expr_lit.lit {
                    lit_int.base10_parse::<i32>()
                } else {
                    Err(Error::new_spanned(
                        expr,
                        "Discriminant must be an integer literal",
                    ))
                }
            }
            Expr::Unary(expr_unary) => {
                if !matches!(expr_unary.op, UnOp::Neg(_)) {
                    return Err(Error::new_spanned(
                        expr_unary.op,
                        "Only unary negation `-` is supported in discriminants",
                    ));
                }
                if let Expr::Lit(expr_lit) = &*expr_unary.expr {
                    if let Lit::Int(lit_int) = &expr_lit.lit {
                        Ok(-lit_int.base10_parse::<i32>()?)
                    } else {
                        Err(Error::new_spanned(
                            &expr_unary.expr,
                            "Expected an integer literal after negation operator",
                        ))
                    }
                } else {
                    Err(Error::new_spanned(
                        &expr_unary.expr,
                        "Expected a literal after negation operator",
                    ))
                }
            }
            _ => Err(Error::new_spanned(
                expr,
                "Unsupported discriminant expression. Must be an integer literal (e.g., `47` or `-1`).",
            )),
        }
    }

    /// Finds and parses the `#[pvn(...)]` attribute on a variant.
    pub fn parse_pvn_attribute(variant: &Variant) -> Result<PvnAttribute> {
        if let Some(attr) = variant
            .attrs
            .iter()
            .find(|attr| attr.path().is_ident("pvn"))
        {
            attr.parse_args::<PvnAttribute>()
        } else {
            Ok(PvnAttribute {
                packets: None,
                data: None,
            })
        }
    }

    /// Creates a human-readable string literal from a variant identifier (e.g., `V1_18_2` -> `"1.18.2"`).
    pub fn humanize_variant_name(ident: &Ident) -> LitStr {
        let name = ident.to_string();
        let humanized = if name == "Any" {
            name
        } else {
            name.strip_prefix('V').unwrap_or(&name).replace('_', ".")
        };
        LitStr::new(&humanized, ident.span())
    }
}
