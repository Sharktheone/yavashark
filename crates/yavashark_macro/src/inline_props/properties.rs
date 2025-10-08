use crate::config::Config;
use crate::inline_props::property::{Kind, Name, Property};
use quote::quote;

pub fn generate_properties(props: &[Property], config: &Config) -> proc_macro2::TokenStream {
    let realm = &config.realm;
    let env = &config.env_path;
    let into_value = &config.into_value;
    let res = &config.res;
    let property_key = &config.property_key;

    let mut prop_items = Vec::with_capacity(props.len());

    for prop in props.iter().filter(|p| p.kind != Kind::Setter) {
        let key = &prop.name;
        let field = &prop.field;

        let partial_get = if prop.partial {
            quote! {
                .get(realm)?
            }
        } else {
            quote! {}
        };

        let get = if prop.copy && !prop.readonly {
            quote! {
                #partial_get .get()
            }
        } else if !prop.copy && !prop.readonly {
            quote! {
                #partial_get .borrow().clone()
            }
        } else {
            quote! {
                #partial_get .clone()
            }
        };

        let value_expr = if prop.kind == Kind::Getter {
            quote! {
                #env::inline_props::Property::Getter(self.#field #get)
            }
        } else {
            quote! {
                {
                let val = #into_value::into_value(self.#field #get);
                #env::inline_props::Property::Value(val)
                }
            }
        };

        match key {
            Name::Str(s) => {
                prop_items.push(quote::quote! {
                    (#property_key::from_static(#s), #value_expr)
                });
            }
            Name::Symbol(sym) => {
                prop_items.push(quote::quote! {
                    (#property_key::from_symbol(#sym.clone()), #value_expr)
                });
            }
        }
    }

    quote::quote! {
        #[inline(always)]
        fn properties(&self, realm: &mut #realm) -> #res<impl ::core::iter::Iterator<Item=(#property_key, #env::inline_props::Property)>> {
            ::core::result::Result::Ok(
                ::core::iter::IntoIterator::into_iter([
                    #(#prop_items),*
                ])
            )
        }
    }
}

pub fn generate_enumerable_properties(
    props: &[Property],
    config: &Config,
) -> proc_macro2::TokenStream {
    let env = &config.env_path;
    let into_value = &config.into_value;
    let res = &config.res;
    let property_key = &config.property_key;
    let realm = &config.realm;

    let mut prop_items = Vec::with_capacity(props.len());

    for prop in props
        .iter()
        .filter(|p| p.kind != Kind::Setter && p.enumerable)
    {
        let key = &prop.name;
        let field = &prop.field;

        let partial_get = if prop.partial {
            quote! {
                .get(realm)?
            }
        } else {
            quote! {}
        };

        let get = if prop.copy && !prop.readonly {
            quote! {
                #partial_get .get()
            }
        } else if !prop.copy && !prop.readonly {
            quote! {
                #partial_get .borrow().clone()
            }
        } else {
            quote! {
                #partial_get .clone()
            }
        };

        let value_expr = if prop.kind == Kind::Getter {
            quote! {
                #env::inline_props::Property::Getter(self.#field #get)
            }
        } else {
            quote! {
                {
                let val = #into_value::into_value(self.#field #get);
                #env::inline_props::Property::Value(val)
                }
            }
        };

        match key {
            Name::Str(s) => {
                prop_items.push(quote::quote! {
                    (#property_key::from_static(#s), #value_expr)
                });
            }
            Name::Symbol(sym) => {
                prop_items.push(quote::quote! {
                    (#property_key::from_symbol(#sym.clone()), #value_expr)
                });
            }
        }
    }

    quote::quote! {
        #[inline(always)]
        fn enumerable_properties(&self, realm: &mut #realm) -> #res<impl ::core::iter::Iterator<Item=(#property_key, #env::inline_props::Property)>> {
            ::core::result::Result::Ok(
                ::core::iter::IntoIterator::into_iter([
                    #(#prop_items),*
                ])
            )
        }
    }
}
