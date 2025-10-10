use crate::config::Config;
use crate::inline_props::property::{Kind, Name, Property};

pub fn generate_keys(props: &[Property], config: &Config) -> proc_macro2::TokenStream {
    let res = &config.res;
    let property_key = &config.property_key;
    let realm = &config.realm;

    let mut prop_items = Vec::with_capacity(props.len());

    for prop in props.iter().filter(|p| p.kind != Kind::Setter) {
        let key = &prop.name;

        match key {
            Name::Str(s) => {
                prop_items.push(quote::quote! {
                    #property_key::from_static(#s)
                });
            }
            Name::Symbol(sym) => {
                prop_items.push(quote::quote! {
                    #property_key::from_symbol(#sym.clone())
                });
            }
        }
    }

    quote::quote! {
        #[inline(always)]
        fn keys(&self, realm: &mut #realm) -> #res<impl ::core::iter::Iterator<Item=(#property_key)>> {
            ::core::result::Result::Ok(
                ::core::iter::IntoIterator::into_iter([
                    #(#prop_items),*
                ])
            )
        }
    }
}

pub fn generate_enumerable_keys(props: &[Property], config: &Config) -> proc_macro2::TokenStream {
    let res = &config.res;
    let property_key = &config.property_key;
    let realm = &config.realm;

    let mut prop_items = Vec::with_capacity(props.len());

    for prop in props
        .iter()
        .filter(|p| p.kind != Kind::Setter && p.enumerable)
    {
        let key = &prop.name;

        match key {
            Name::Str(s) => {
                prop_items.push(quote::quote! {
                    #property_key::from_static(#s)
                });
            }
            Name::Symbol(sym) => {
                prop_items.push(quote::quote! {
                    #property_key::from_symbol(#sym.clone())
                });
            }
        }
    }

    quote::quote! {
        #[inline(always)]
        fn enumerable_keys(&self, realm: &mut #realm) -> #res<impl ::core::iter::Iterator<Item=(#property_key)>> {
            ::core::result::Result::Ok(
                ::core::iter::IntoIterator::into_iter([
                    #(#prop_items),*
                ])
            )
        }
    }
}
