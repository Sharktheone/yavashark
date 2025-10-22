use proc_macro2::Ident;
use syn::{Expr, Type};

pub struct Property {
    pub id: Ident,
    pub name: Option<Expr>,
    pub ty: Type,
    pub required: bool,
}

pub fn parse_properties(fields: &mut syn::Fields) -> syn::Result<Vec<Property>> {
    let syn::Fields::Named(fields) = fields else {
        return Err(syn::Error::new_spanned(
            fields,
            "Expected named fields",
        ));
    };


    let mut properties = Vec::with_capacity(fields.named.len());

    for field in fields.named.iter_mut() {
        let mut property_name = None;
        let mut required = false;
        

        for attr in &field.attrs {
            if attr.meta.path().is_ident("prop") {
                let n = match attr.parse_args().map_err(|e| syn::Error::new(e.span(), e)) {
                    Ok(n) => n,
                    Err(e) => {
                        return Err(syn::Error::new_spanned(
                            attr,
                            format!("Failed to parse property name: {}", e),
                        ));
                    }
                };

                property_name = Some(n);
            }
            
            if attr.meta.path().is_ident("required") {
                required = true;
            }
        }
        
        
        //TODO: hacky
        field.attrs.retain(|attr| {
            !attr.meta.path().is_ident("prop") && !attr.meta.path().is_ident("required")
        });

        let id = field.ident.clone().expect("Expected named fields");

        properties.push(Property {
            id,
            name: property_name,
            ty: field.ty.clone(),
            required,
        });
    }

    Ok(properties)
}