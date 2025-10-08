use crate::config::Config;
use crate::inline_props::property::{Name, Property};
use quote::quote;

pub fn generate_contains_property(props: &[Property], config: &Config) -> proc_macro2::TokenStream {
    let internal_property_key = &config.internal_property_key;
    let res = &config.res;

    let mut string_arms = Vec::with_capacity(props.len());
    let mut symbols = Vec::new();

    for prop in props {
        let key = &prop.name;

        let value_expr = quote! {
            return ::core::result::Result::Ok(true);
        };

        match key {
            Name::Str(s) => {
                string_arms.push(quote::quote! {
                    #s => {
                        #value_expr
                    }
                });
            }
            Name::Symbol(sym) => {
                symbols.push(quote::quote! {
                    if symbol == #sym {
                        #value_expr
                    }
                });
            }
        }
    }

    let str_check = if !string_arms.is_empty() {
        quote::quote! {
            #internal_property_key::String(str) => {
                match str.as_str() {
                    #(#string_arms)*
                    _ => {}
                }
            },
        }
    } else {
        quote::quote! {}
    };

    let symbol_check = if !symbols.is_empty() {
        quote::quote! {
            #internal_property_key::Symbol(symbol) => {
                #(#symbols)*
            },
        }
    } else {
        quote::quote! {}
    };

    quote::quote! {
        #[inline(always)]
        fn contains_property(&self, key: &#internal_property_key) -> #res<bool> {
            match key {
                #str_check
                #symbol_check
                _ => {}

            }


            ::core::result::Result::Ok(false)
        }
    }
}
