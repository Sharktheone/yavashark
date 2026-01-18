use crate::config::Config;
use crate::obj::args::{Direct, DirectItem};
use darling::ast::NestedMeta;
use darling::FromMeta;
use proc_macro::TokenStream as TokenStream1;
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};

pub fn custom_props(attrs: TokenStream1, item: TokenStream1) -> TokenStream1 {
    let conf = Config::new(Span::call_site());
    let error = &conf.error;

    let attr_args = match NestedMeta::parse_meta_list(attrs.clone().into()) {
        Ok(v) => v,
        Err(e) => {
            return TokenStream1::from(darling::Error::from(e).write_errors());
        }
    };

    let args = match Direct::from_list(&attr_args) {
        Ok(args) => args,
        Err(e) => return e.write_errors().into(),
    };

    let direct = args.fields;

    if direct.is_empty() {
        return item;
    }

    let mut item: syn::ItemImpl = syn::parse_macro_input!(item);

    let value = &conf.value;
    let variable = &conf.variable;
    let obj_prop = &conf.object_property;
    let realm = &conf.realm;
    let internal_property_key = &conf.internal_property_key;

    let properties_define = match_prop(&direct, Act::Set, &conf);

    item.items.push(syn::parse_quote! {
        fn define_property(&self, name: #internal_property_key, value: #value, &mut #realm) -> Result<(), #error> {
            let mut inner = self.get_inner_mut();
            #properties_define

            drop(inner);

            self.get_wrapped_object().define_property(name, value)
        }
    });

    let properties_variable_define = match_prop(&direct, Act::SetVar, &conf);

    item.items.push(syn::parse_quote! {
        fn define_property_attribute(&self, name: #internal_property_key, value: #variable, realm: &mut #realm) -> Result<(), #error> {
            let mut inner = self.get_inner_mut();
            #properties_variable_define

            drop(inner);

            self.get_wrapped_object().define_property_attributes(name, value, realm)
        }
    });

    let properties_resolve = match_prop(&direct, Act::None, &conf);

    item.items.push(syn::parse_quote! {
        fn resolve_property(&self, name: & #value) -> Result<Option<#obj_prop>, #error> {
            let inner = self.get_inner();
            #properties_resolve

            drop(inner);

            self.get_wrapped_object().resolve_property(name)
        }
    });

    let properties_get = match_prop(&direct, Act::Get, &conf);

    item.items.push(syn::parse_quote! {
        fn get_property(&self, name: & #value) -> Result<Option<#obj_prop>, #error> {
            let inner = self.get_inner();
            #properties_get

            drop(inner);

            self.get_wrapped_object().get_property(name)
        }
    });

    let properties_delete = match_prop(&direct, Act::Delete, &conf);

    item.items.push(syn::parse_quote! {
        fn delete_property(&self, name: &Value) -> Result<Option<Value>, #error> {
            let mut inner = self.get_inner_mut();
            #properties_delete

            drop(inner);

            self.get_wrapped_object().delete_property(name)
        }
    });

    let properties_contains = match_prop(&direct, Act::Contains, &conf);

    item.items.push(syn::parse_quote! {
        fn contains_key(&self, name: & #value) -> Result<bool, #error> {
            let inner = self.get_inner();
            #properties_contains

            drop(inner);

            self.get_wrapped_object().contains_key(name)
        }
    });

    let properties = match_list(&direct, List::Properties, &conf);

    item.items.push(syn::parse_quote! {
        fn properties(&self) -> Result<Vec<(#value, #value)>, #error> {
            let mut props = self.get_wrapped_object().properties()?;
            let inner = self.get_inner();

            #properties

            drop(inner);

            Ok(props)
        }
    });

    let keys = match_list(&direct, List::Keys, &conf);

    item.items.push(syn::parse_quote! {
        fn keys(&self) -> Result<Vec<#value>, #error> {
            let mut keys = self.get_wrapped_object().keys()?;
            let inner = self.get_inner();

            #keys

            drop(inner);

            Ok(keys)
        }
    });

    let values = match_list(&direct, List::Values, &conf);

    item.items.push(syn::parse_quote! {
        fn values(&self) -> Result<Vec<#value>, #error> {
            let mut values = self.get_wrapped_object().values()?;
            let inner = self.get_inner();

            #values

            drop(inner);

            Ok(values)
        }
    });

    let clear = match_list(&direct, List::Clear, &conf);

    item.items.push(syn::parse_quote! {
        fn clear_values(&self) -> Result<(), #error> {
            self.get_wrapped_object().clear_values()?;
            let mut inner = self.get_inner_mut();

            #clear

            drop(inner);

            Ok(())
        }
    });

    item.to_token_stream().into()
}

#[derive(Debug, Eq, PartialEq)]
pub enum Act {
    Get,
    // RefMut,
    None,
    Set,
    SetVar,
    Contains,
    Delete,
}

pub fn match_prop(properties: &[DirectItem], r: Act, config: &Config) -> TokenStream {
    let value_path = &config.value;
    let internal_property_key = &config.internal_property_key;
    let define_property_result = &config.define_property_result;
    let property = &config.property;

    let mut match_properties_define = TokenStream::new();
    let match_non_string = TokenStream::new();

    for item in properties {
        let field = &item.field;

        let act = match r {
            Act::Get => quote! {Some(inner.#field.clone().into())},
            // Act::RefMut => quote! {Some(&mut self.#field.value)},
            Act::None => quote! {Some(inner.#field.clone().into())},
            Act::Set => quote! {{inner.#field = value.into(); #define_property_result::Handled}},
            Act::SetVar => quote! {{inner.#field = value.into(); #define_property_result::Handled}},
            Act::Contains => quote! {true},
            Act::Delete => quote! {
                {
                    let old = inner.#field.value.copy();
                    inner.#field.value = #value_path::Undefined;
                    Some(#property::Value(old.into(), inner.#field.attributes)) //TODO: this is not correct for setters / getters
                }
            },
        };

        if let Some(rename) = &item.rename {
            let expanded = if matches!(r, Act::Set | Act::SetVar) {
                quote! {
                    Some(stringify!(#rename)) => {
                        #act;
                        return Ok(#define_property_result::Handled);
                    }
                }
            } else {
                quote! {
                    Some(stringify!(#rename)) => {
                        return Ok(#act);
                    }
                }
            };

            match_properties_define.extend(expanded); //TODO: we currently don't have a way to set up a non string field
            continue;
        }

        let expanded = quote! {
            Some(stringify!(#field)) =>  {
                return Ok(#act);
            }
        };

        match_properties_define.extend(expanded);
    }

    if !match_properties_define.is_empty() {
        match_properties_define = quote! {
            if let #internal_property_key::String(name) = &name {
                match name.as_str() {
                    #match_properties_define
                    _ => {}
                }
            }
        };
    }

    if !match_non_string.is_empty() {
        match_properties_define = quote! {
            #match_properties_define

            match name {
                #match_non_string
                _ => {}
            }
        };
    }

    match_properties_define
}

pub enum List {
    Properties,
    Keys,
    Values,
    Clear,
}

pub fn match_list(properties: &[DirectItem], r: List, config: &Config) -> TokenStream {
    let value = &config.value;
    let property_key = &config.property_key;
    let mut add = TokenStream::new();

    for item in properties {
        let field = &item.field;

        let name = if let Some(rename) = &item.rename {
            quote! { #property_key::from(stringify!(#rename)) }
        } else {
            quote! {
                #property_key::from(stringify!(#field))
            }
        };

        let act = match r {
            List::Properties => {
                quote! {props.push((#name, inner.#field.value.copy().into()));}
            }
            List::Keys => quote! {keys.push(#name);},
            List::Values => quote! {values.push(inner.#field.value.copy().into());},
            List::Clear => quote! {inner.#field.value = #value::Undefined;},
        };

        add.extend(act);
    }

    add
}
