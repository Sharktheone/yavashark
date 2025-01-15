use crate::config::Config;
use proc_macro2::Ident;
use quote::{quote, ToTokens};
use syn::Field;

pub struct MutableRegion {
    direct: Vec<Ident>,
    custom: Vec<Field>,
    name: Ident,
}

impl MutableRegion {
    pub(crate) fn with(direct: Vec<Ident>, custom: Vec<Field>, name: Ident) -> Self {
        Self {
            direct,
            custom,
            name,
        }
    }

    pub fn full_name(&self) -> Ident {
        Ident::new(&format!("Mutable{}", self.name), self.name.span())
    }

    pub fn generate(&self, config: &Config, object: bool) -> proc_macro2::TokenStream {
        let full_name = self.full_name();

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
            #[derive(Debug, PartialEq, Eq)]
            pub struct #full_name {
                #object
                #(#direct)*
                #(#custom)*
            }
        }
    }
}
