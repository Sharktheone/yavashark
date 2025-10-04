use crate::config::Config;
use crate::inline_props::property::{Kind, Name, Property};

pub fn generate_get_property(
    props: &[Property],
    config: &Config,
) -> proc_macro2::TokenStream {

    let internal_property_key = &config.internal_property_key;
    let realm = &config.realm;
    let env = &config.env_path;
    let into_value = &config.into_value;
    let res = &config.res;


    let mut string_arms = Vec::with_capacity(props.len());
    let mut symbols = Vec::new();



    for prop in props.iter().filter(|p| matches!(p.kind, Kind::Property | Kind::Getter)) {
        let key = &prop.name;
        let ty = &prop.ty;
        let field = &prop.field;

        let partial_get = if prop.partial {
            quote::quote! {
                .get(realm)?
            }
        } else {
            quote::quote! {}
        };

        let value_expr = if prop.readonly {
            if prop.kind == Kind::Getter {
                quote::quote! {
                    return ::core::result::Result::Ok(::core::option::Option::Some(#env::inline_props::Property::Getter(self.#field #partial_get .clone())));
                }
            } else {
                quote::quote! {
                    let val = #into_value::into_value(self.#field #partial_get.clone());
                    return ::core::result::Result::Ok(::core::option::Option::Some(#env::inline_props::Property::Value(val)));
                }
            }
        } else if prop.copy {
            quote::quote! {
                let val = #into_value::into_value(self.#field #partial_get .get());
                return ::core::result::Result::Ok(::core::option::Option::Some(#env::inline_props::Property::Value(val)));
            }
        } else {
            quote::quote! {
                let val = self.#field #partial_get.borrow().clone();
                let val = #into_value::into_value(val);

                return ::core::result::Result::Ok(::core::option::Option::Some(#env::inline_props::Property::Value(val)));
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
        fn get_property(&self, key: &#internal_property_key, realm: &mut #realm) -> #res<::core::option::Option<#env::inline_props::Property>> {
            match key {
                #str_check
                #symbol_check
                _ => {}

            }


            ::core::result::Result::Ok(::core::option::Option::None)
        }
    }
}