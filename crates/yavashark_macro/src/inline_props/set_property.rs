use proc_macro2::TokenStream;
use quote::quote;
use crate::config::Config;
use crate::inline_props::property::{Kind, Name, Property};

pub fn generate_set_property(
    props: &[Property],
    config: &Config,
) -> TokenStream {
    let internal_property_key = &config.internal_property_key;
    let value = &config.value;
    let realm = &config.realm;
    let env = &config.env_path;
    let res = &config.res;
    let from_value_output = &config.from_value_output;
    
    
    let mut string_arms = Vec::with_capacity(props.len());
    let mut symbols = Vec::new();
    
    for prop in props.iter().filter(|p| matches!(p.kind, Kind::Property | Kind::Setter)) {
        let key = &prop.name;
        let ty = &prop.ty;
        let field = &prop.field;

        let partial_get = if prop.partial {
            quote! {
                .get(realm)?
            }
        } else {
            quote! {}
        };
        
        let value_expr = if prop.readonly {
            if prop.kind == Kind::Setter {
                quote::quote! {
                return Ok(#env::inline_props::UpdatePropertyResult::Setter(self.#field #partial_get .clone(), value));
                }
            } else {
                quote::quote! {
                    return Ok(#env::inline_props::UpdatePropertyResult::ReadOnly);
                }
            }
        } else if prop.copy {
            quote::quote! {
                self.#field #partial_get .set(<#ty as #from_value_output>::from_value_out(value, realm)?);
                return Ok(#env::inline_props::UpdatePropertyResult::Handled);
            }
        } else {
            quote::quote! {
                *self.#field #partial_get .borrow_mut() = <#ty as #from_value_output>::from_value_out(value, realm)?;
                return Ok(#env::inline_props::UpdatePropertyResult::Handled);
            }
        };

        match key {
            Name::Str(s) => {
                string_arms.push(quote::quote! {
                    #s => {
                        #value_expr
                    }
                });
            },
            Name::Symbol(sym) => {
                symbols.push(quote::quote! {
                    if symbol == #sym {
                        #value_expr
                    }
                });
            },
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
        fn set_property(&self, key: &#internal_property_key, value: #value, realm: &mut #realm) -> #res<#env::inline_props::UpdatePropertyResult> {
            match key {
                #str_check
                #symbol_check
                _ => {}
                
            }

            
            ::core::result::Result::Ok(#env::inline_props::UpdatePropertyResult::NotHandled(value))
        }
    }
}