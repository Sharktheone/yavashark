use crate::config::Config;
use proc_macro2::Ident;
use quote::{quote, ToTokens};
use syn::Field;

struct MutableRegion {
    direct: Vec<Ident>,
    custom: Vec<Field>,
    name: Ident,
}

impl MutableRegion {
    fn new(name: Ident) -> Self {
        Self {
            direct: vec![],
            custom: vec![],
            name,
        }
    }

    fn add_direct(&mut self, field: Ident) {
        self.direct.push(field);
    }

    fn add_custom(&mut self, field: Field) {
        self.custom.push(field);
    }

    fn generate(&self, config: &Config, object: bool) -> proc_macro2::TokenStream {
        let name = &self.name;
        let full_name = format!("Mutable{}", name);

        let prop = &config.object_property;

        let custom = self.custom.iter().map(|field| field.to_token_stream());

        let direct = self.direct.iter().map(|field| {
            quote! {
                #field: #prop,
            }
        });
        
        let mut_object = &config.mut_object;
        
        let object = if object {
            quote! {
                pub object: #mut_object,
            }
        } else {
            quote! {}
        };

        quote! {
            pub struct #full_name {
                #object
                #(#direct)*
                #(#custom)*
            }
        }
    }
}
