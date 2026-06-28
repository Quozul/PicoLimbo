use syn::{Data, DeriveInput, Fields, Field, Expr};

pub fn get_named_fields(input: &DeriveInput) -> &syn::punctuated::Punctuated<Field, syn::Token![,]> {
    if let Data::Struct(data) = &input.data {
        if let Fields::Named(fields) = &data.fields {
            return &fields.named;
        }
    }
    unimplemented!()
}

pub fn get_pvn_attribute(field: &Field) -> Option<Expr> {
    field.attrs.iter().find_map(|attr| {
        if attr.path().is_ident("pvn") {
            Some(attr.parse_args::<Expr>().unwrap())
        } else {
            None
        }
    })
}
