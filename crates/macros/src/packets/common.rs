use quote::quote;
use syn::{Attribute, Ident};

pub struct VersionConstraint {
    pub min: Option<Ident>,
    pub max: Option<Ident>,
}

impl VersionConstraint {
    pub fn from_attribute(attr: &Attribute) -> Option<Self> {
        if !attr.path().is_ident("protocol_version") {
            return None;
        }

        let mut constraint = VersionConstraint {
            min: None,
            max: None,
        };

        if attr
            .parse_nested_meta(|meta| {
                if meta.path.is_ident("min") {
                    let _ = meta.value()?;
                    constraint.min = Some(meta.input.parse::<Ident>()?);
                } else if meta.path.is_ident("max") {
                    let _ = meta.value()?;
                    constraint.max = Some(meta.input.parse::<Ident>()?);
                }
                Ok(())
            })
            .is_err()
        {
            // Ignore parsing errors for now; they will be caught by the compiler
        }

        Some(constraint)
    }

    pub fn generate_check(&self) -> proc_macro2::TokenStream {
        match (&self.min, &self.max) {
            (Some(min), Some(max)) => {
                quote! { protocol_version.between_inclusive(ProtocolVersion::#min, ProtocolVersion::#max) }
            }
            (Some(min), None) => {
                quote! { protocol_version.is_after_inclusive(ProtocolVersion::#min) }
            }
            (None, Some(max)) => {
                quote! { protocol_version.is_before_inclusive(ProtocolVersion::#max) }
            }
            (None, None) => {
                quote! { true }
            }
        }
    }
}
