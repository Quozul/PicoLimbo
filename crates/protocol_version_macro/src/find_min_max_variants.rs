use crate::parsed_variant::ParsedVariant;
use syn::{Error, Ident, Result};

/// Finds the oldest and latest "real" (non-negative) variants.
pub fn find_min_max_variants<'a>(
    variants: &'a [ParsedVariant<'a>],
    enum_span: proc_macro2::Span,
) -> Result<(&'a Ident, &'a Ident)> {
    let real_variants: Vec<_> = variants
        .iter()
        .filter(|v| v.discriminant_value >= 0)
        .collect();

    if real_variants.is_empty() {
        return Err(Error::new(
            enum_span,
            "At least one real (non-negative) variant is required to determine oldest() and latest()",
        ));
    }

    let min_variant = real_variants
        .iter()
        .min_by_key(|v| v.discriminant_value)
        .unwrap();
    let max_variant = real_variants
        .iter()
        .max_by_key(|v| v.discriminant_value)
        .unwrap();

    Ok((min_variant.ident, max_variant.ident))
}
