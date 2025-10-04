use quote::quote;
use crate::config::Config;
use crate::inline_props::property::{Kind, Name, Property};

pub fn generate_values(
    props: &[Property],
    config: &Config,
) -> proc_macro2::TokenStream {

    let env = &config.env_path;
    let into_value = &config.into_value;
    let res = &config.res;


    let mut prop_items = Vec::with_capacity(props.len());



    for prop in props.iter().filter(|p| p.kind != Kind::Setter) {
        let field = &prop.field;

        let get = if prop.copy && !prop.readonly {
            quote! {
                .get()
            }
        } else if !prop.copy && !prop.readonly {
            quote! {
                .borrow().clone()
            }
        } else {
            quote! {
                .clone()
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

        
        prop_items.push(value_expr);
    }




    quote::quote! {
        #[inline(always)]
        fn values(&self) -> #res<impl ::core::iter::Iterator<Item=#env::inline_props::Property>> {
            ::core::result::Result::Ok(
                ::core::iter::IntoIterator::into_iter([
                    #(#prop_items),*
                ])
            )
        }
    }
}


pub fn generate_enumerable_values(
    props: &[Property],
    config: &Config,
) -> proc_macro2::TokenStream {

    let env = &config.env_path;
    let into_value = &config.into_value;
    let res = &config.res;


    let mut prop_items = Vec::with_capacity(props.len());



    for prop in props.iter().filter(|p| p.kind != Kind::Setter && p.enumerable) {
        let field = &prop.field;


        let get = if prop.copy && !prop.readonly {
            quote! {
                .get()
            }
        } else if !prop.copy && !prop.readonly {
            quote! {
                .borrow().clone()
            }
        } else {
            quote! {
                .clone()
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


        prop_items.push(value_expr);
    }




    quote::quote! {
        #[inline(always)]
        fn enumerable_values(&self) -> #res<impl ::core::iter::Iterator<Item=#env::inline_props::Property>> {
            ::core::result::Result::Ok(
                ::core::iter::IntoIterator::into_iter([
                    #(#prop_items),*
                ])
            )
        }
    }
}
