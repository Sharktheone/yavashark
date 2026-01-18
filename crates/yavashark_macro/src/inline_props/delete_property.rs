use crate::config::Config;
use crate::inline_props::property::{Name, Property};
use proc_macro2::TokenStream;

pub fn generate_delete_property(props: &[Property], config: &Config) -> TokenStream {
    let res = &config.res;
    let realm = &config.realm;
    let internal_property_key = &config.internal_property_key;

    let mut string_arms = Vec::with_capacity(props.len());
    let mut symbols = Vec::new();

    let mut prop_items = Vec::new();
    let mut has_configurable = false;
    let mut config_idx = 0usize;
    let mut write_idx = 0usize;

    for prop in props {
        if !prop.configurable {
            if prop.configurable {
                config_idx += 1;
            }
            continue;
        }

        let c = config_idx;
        config_idx += 1;

        let w = write_idx;
        if !prop.readonly {
            write_idx += 1;
        }

        has_configurable = true;

        let key = &prop.name;

        let value_expr = if prop.readonly {
            quote::quote! {
            self.__deleted_properties.set(self.__deleted_properties.get() | (1 << #c));
            //TODO: probably we should alco clear the value, but that might be a bit more difficult
            return ::core::result::Result::Ok(true);
            }
        } else {
            quote::quote! {
                if (self.__written_properties.get() & (1 << #w)) != 0 {
                    return ::core::result::Result::Ok(false);
                }

                self.__deleted_properties.set(self.__deleted_properties.get() | (1 << #c));
                return ::core::result::Result::Ok(true);
            }
        };

        match key {
            Name::Str(s) => {
                string_arms.push(quote::quote! {
                    Some(#s) => {
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

        prop_items.push(value_expr);
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

    if !has_configurable {
        quote::quote! {
            #[inline(always)]
            fn delete_property(
                &self,
                key: &#internal_property_key,
                realm: &mut #realm
            ) -> #res<bool> {
                ::core::result::Result::Ok(false)
            }
        }
    } else {
        quote::quote! {
            #[inline(always)]
            fn delete_property(
                &self,
                key: &#internal_property_key,
                realm: &mut #realm
            ) -> #res<bool> {
                match key {
                    #str_check
                    #symbol_check
                    _ => {}
                }

                ::core::result::Result::Ok(false)
            }
        }
    }
}
